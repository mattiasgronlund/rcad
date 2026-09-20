## 44. What stands in the door — the unit after M5, recorded 2026-09-03

*Written before any lane of this unit was dispatched, and deliberately not in
§42's shape. It settles what the unit is, what it is called, four decisions it
inherits unsettled, and the corrections planning made to §42, to §43 and to
itself. Every file and line it cites was checked against the tree at `dc4c632`.*

### 44.1 The unit has a name because "M6" already has three meanings

Three documents in this repository use the name M6 for three different bodies of
work, and nobody reconciled them:

| where | what it says M6 is |
|---|---|
| roadmap §12 | local operations, fillets first |
| §43.10's recorded prediction | corner slop, criterion 8, and the fuse's unjudged `expect`s — no fillets in it |
| §43.8's fourteen handed-forward items | a third list, overlapping neither cleanly |

**Decision: this unit is called *the door*, and the roadmap's M6 number is left
alone.** Roadmap §12 keeps M6 for local operations; nothing here is amended, per
principle 6 — the disagreement between roadmap and tree is deliberate and lives
in this file. The name comes from §43.10's own closing sentence: *"the door is
open, and three known things stand in it."* This unit removes two of the three.

**§43.10's prediction is inherited by this unit and will be scored at its closing
gate**, not at the roadmap's M6. It was written about this work — corner slop
first, criterion 8 second, the fuse's `expect`s third — so scoring it anywhere
else would score it against a different milestone.

### 44.2 There is no a-priori criteria list, and that is the change

§42.12 wrote fourteen exit criteria before any M5 lane ran. §43.2 scored them: 8
met, 2 in part, 4 unmet. **The instrument was frozen at the moment of least
knowledge, and it misread the work twice, in opposite directions.**

- **It deferred two things that turned out to be mandatory.** §42.13 put the cone
  and the torus behind "the spike's count". Both were built, and they moved the
  import ceiling 25 and 10 points (§43.1). The criteria list said nothing about
  the largest single item in the milestone.
- **It priced a lane that never ran.** §42.12's "three inherited defects" lane
  carried criteria 11, 12 and 13. It was never dispatched, and it took three of
  the four unmet criteria with it. Those three are not a measurement of M5; they
  are a measurement of a dispatch that did not happen.
- **Its own worked example went stale.** Criterion 4 said *"a file holding a cone
  is refused by the name the file gives it"*, and M5 built the cone (§43.7).

**So: keep the closing gate, drop the opening gate's criteria.** §43.8 was
written knowing the tree — every item with its evidence, its file and line, and
its named consumer — and is a strictly better commitment document than a list
written before the tree existed.

**What replaces the criteria, and the risk that takes on.** §30.3's warning
stands: *a criterion that cannot fail is not a criterion*, and dropping the list
without replacing its failure mode would be a gate that cannot fail. What
replaces it is §44.10's verification list, in which **every line names something
in the tree that changes state** — a test that must start failing, a pinned list
of filenames that must move, a count that must be published. §43.9's own finding
about criterion 10 is the model: a rate that is printed and not asserted is not a
criterion, and a list of filenames that must match is.

### 44.3 The sentence this unit earns

**A file whose corners are as close as an `f64` can put them is no longer refused
for being off.**

Ninety of the 367 corpus files are refused because a corner is not on the curved
face the file says it is on, where **the file wrote the nearest `f64` there is**
and no writer could have done better (§43.5). That is a *gap* in CLAUDE.md's
sense — the answer exists and the arithmetic could not reach it — and the flat
case is already solved in the tree, so it has a working precedent rather than a
research problem. The second sentence, worth 75 more files, is that a face
bounded by an arc and its chord is a face.

### 44.4 The scope, and the test every item passed

**The test: a named consumer AND a settled decision.** §13 rule 3 supplies the
first half; the second half is what keeps a lane from spending its 45 minutes
making a decision that is the milestone owner's.

| stage | item | consumer | §43.8 |
|---|---|---|---|
| 0 | the `ModelData` deserialize asymmetry | a `Model` off the wire can switch off the certified check | 13, widened |
| 1 | the curved writer's-rounding budget | ≤ 88 KiCad + 2 NIST files | 12, its gap half |
| 1 | criterion 8 — do two solids share space | roadmap §7.4; unstarted for two milestones | §43.2 criterion 8 |
| 1 | `arrange.rs`'s tangent direction-recovery gap | `degenerate_curved_fuses.rs:1539-1547` | — |
| 1 | criterion 10's pinning | the unit's own headline number | 10, and §43.9's finding |
| 2 | two-step wires | 75 of 367 files, 16 of the 44 NIST | 1, its topo half |
| 2 | the `validate.rs:436` skip — measure first | two doc comments that appear to disagree | — |
| 2 | criterion 12 — the mesher's round-trip charge | §41.11 item 19 | §43.2 criterion 12 |
| 2 | `SolidBuilder::build` takes a placement | criterion 11's other half | 11 |
| 2 | the seven `KnotVector` getters — argue, do not change | a decisions.md entry and seven tests | 2 |
| 2 | the cheap closures | four small ones, listed in §44.9 | 8, 9, and the two unnumbered |

### 44.5 Lane 0 — the deserialize asymmetry, and a second dispatch nobody specified

**It runs first and alone.** It is the only item on the list that can make a
certified answer *wrong* — everything else is a refusal that is honest today —
and it owns `validate.rs`, which four of five candidate lanes want.

**The asymmetry.** Every type on `Model`'s deserialize path either has a door
that can refuse, or does not:

| type | serde attribute | guard runs on deserialize |
|---|---|---|
| `Scale` | `#[serde(try_from = "(Unit, f64)")]` (`rcad-geom/src/scale.rs:56`) | **yes** — through `Scale::new` (`:78-83`) |
| all five surfaces, `Axis`, `Isometry`, `HalfSpace`, `Segment`, `Deflection`, `KnotVector` | `try_from` | **yes** |
| `Approximation` | plain derive (`rcad-predicates/src/provenance.rs:10`) | **no** — `Approximation::new`'s assert (`:30-35`) is never called |
| `Point`, `Vector` | `#[serde(from = "[f64; 3]")]` (`rcad-geom/src/point.rs:26`, `:170`) | **no**, and `from` cannot refuse |

A grep for `serde(try_from` and `serde(from` over `rcad-geom/src` returns sixteen
attributes and **the two `from`s are `Point` and `Vector`**, so with
`Approximation` that is the complete list of three.

**What the `Approximation` hole costs.** `not_provably_off`
(`rcad-topo/src/validate.rs:1277-1289`) filters magnitudes with `|by| *by > 0.0`,
so an **infinite** magnitude passes the filter, widens the corner into an entire
interval, and makes the on-surface tests at `validate.rs:193` and `:1226`
vacuously true. A `Model` off the wire can switch off the certified check and
`validate` returns `Ok`. `check_approximation_ordering` (`validate.rs:563`)
compares magnitudes with `<` and never asks about sign or finiteness. The
approximation sits on the deserialize path three times, on `VertexData`,
`EdgeData` and `FaceData`. Two fuzz targets already knock on this door —
`fuzz/fuzz_targets/no_model_makes_the_writer_panic.rs:25` and
`two_solids_fused_by_a_stranger.rs:70` — and assert nothing about it.

**What the `Point` hole costs is smaller, and the fix is not a door.** A
non-finite vertex coordinate passes `Deserialize for Model`
(`rcad-topo/src/serialize.rs:105-121`), which runs `check_keys_resolve` and
`check_shape_is_possible` and nothing else. It is caught by `Model::validate` →
`Invalid::VertexNotFinite` (`validate.rs:142-144`), which is not on that path.

**Decision: `Point` and `Vector` keep `from`.** The infallible conversion was
chosen deliberately, a fallible branch on every coordinate in the kernel is a
real cost, and the condition is one a sweep already implements. The fix is to
**extend `check_shape_is_possible` (`validate.rs:125-138`) to ask the finiteness
question `check_vertices` already asks**.

**And that decision exposes the finding this section records in its own right.**
The model has **two validation dispatches and no rule saying what belongs in the
cheap one**:

| | where | what it does today |
|---|---|---|
| `Model::validate` | `validate.rs:79-91` | eleven checks, takes a `&Policy` and a `ProvenanceLog`, asks arithmetic |
| `check_shape_is_possible` | `validate.rs:125-138` | three checks — a face has wires, a wire closes, an edge is walked twice — no arithmetic, no policy |

`check_shape_is_possible` is what `Deserialize` runs, so it is the **untrusted
import boundary**, and its membership was decided one panic at a time: §43.3 shows
five of its eleven rows closed by adding checks here. Nobody has written down what
the rule is.

**The rule this unit adopts, and the lane applies:** *`check_shape_is_possible`
holds every check that needs no `Policy`, no arithmetic beyond `is_finite`, and no
sweep over pairs — because those are exactly the checks a public method is
entitled to take for granted, and a `Model` that fails one of them is not a model
at any strictness.* `Invalid::VertexNotFinite` qualifies and is the first check
moved by it. §43.8 item 10 — a `Model` with one face used twice deserializes and
then crashes the fuse — is the next candidate the rule reaches, and the lane
should say whether counting face uses is a sweep over pairs or not.

**The lane's deliverable, in four parts:**

1. Give `Approximation` the guard `Scale` has: copy the `try_from` pattern from
   `scale.rs:56` / `:115-121` verbatim, routed through `Approximation::new`'s
   condition returned as an `Err` rather than asserted.
2. Make `check_approximation_ordering` test sign and finiteness, not only order.
3. Extend `check_shape_is_possible` per the rule above.
4. **Audit every remaining field of `ModelData` (`rcad-topo/src/model.rs:727-735`)
   and every arena payload type, and publish the table** — whatever else it turns
   up. The audit found its second hole during planning, which is the reason it is
   a deliverable and not a checkbox.

### 44.6 The 90/76 split, and option A's D trigger, scored

§43.7 records §42.9's fork as settled: **option A, refuse by name, nothing is
healed.** That stands and is not reopened. What was never done is the thing §42.9
asked for in the same paragraph.

**§42.9 wrote: *"D is the fallback if A refuses too much to be usable."* §43.8
item 12 silently replaced D with C, and nobody scored the trigger.** A fallback
with no condition attached is not a fallback. So it is scored here.

**The split, from §43.5.** Of the 166 files with a corner off a curved face:

| | files | how far off | limit or gap |
|---:|---|---|---|
| **90** | 88 KiCad + 2 NIST | inside one coordinate spacing | **gap** — the file wrote the nearest `f64`; the arithmetic could not reach it |
| **76** | 73 KiCad + 3 NIST | out to 20 777 spacings (13 169 in NIST) | **limit for this unit** — a modelling tolerance somebody chose, which no arithmetic fixes |

**The reading that suggests itself is not a trigger at all.** *"14.2% is too low,
so fire D"* cannot be one, because a rate can be low for reasons healing cannot
touch, and here it is. Of the 367 files, healing reaches the **76** and
nothing else: the 90 are arithmetic, the 75 two-step wires are a topology rule,
and the 68 B-spline files are a missing surface kind. **A perfect healer bolted on
today buys 76 files out of 321 refused, and leaves the other 245 exactly where
they are.**

**The trigger written so it can fire.** *D fires when, after the named gaps close,
more than half the still-refused files are refused for a miss larger than `f64`
rounding.* Scored on today's numbers: 277 KiCad files are refused, the
curved-rounding gap unblocks **at most 88** and the two-step-wire gap **at most
59**, so at best **130 stay refused**. Of those, the healable ones number between
**65 and 73** — 73 is §43.5's split, and up to 8 of them may be among the 59,
because §43.5's 161 exceeds by 8 the 153 whose *first* refusal the corner miss was.
That is **50% to 56%.**

**So the trigger sits exactly on its own boundary, and that is the finding.** It
is not fired today; it is not comfortably unfired either, and it becomes
decidable only when the rate-pinning lane publishes a per-file refusal reason.
**Neither this section nor a lane resolves it — the closing gate re-scores it
against measured per-file data**, which is the point of writing the condition down
as a fraction rather than as a rate.

**A correction that changes option C's price, and it is this section's largest.**
§42.9 and §43.8 item 12 both say C needs *"a new `Cause` variant and a `Measures`
arm"*. **Both halves are false, demonstrated in the tree:**

- `Cause::ImportedUncertainty` **already exists** and its doc already says exactly
  this — *"The data arrived approximate — an imported file that did not close, or
  coordinates rounded by the exporter"* (`rcad-predicates/src/provenance.rs:72-74`).
  It is **constructed nowhere in `crates/*/src` outside test modules**;
  `rcad-step` never sets an approximation on anything.
- `Measures::Length` **already exists** (`rcad-predicates/src/policy.rs:158`), and
  an imported corner's slop is a length in the model's own unit, which is what
  `Approximation::new` takes.

**So C is one decision and no new mechanism.** The whole of what stands between
the tree and option C is the sentence §42.9 and `rcad-step/src/refusal.rs:339-348`
both name and neither answers: *what is an imported corner's slop?* — with the
constraint, which both state correctly, that a tag equal to the miss makes the
check pass by construction. **This unit does not answer it**; it closes the 90
that do not need it answered, and hands the 76 forward with the price corrected
downward.

### 44.7 The corrected fillet price

Fillets are out of scope. **The reason given for that in planning was wrong, and
the record has to carry the corrected price so the next unit can cost it
honestly.**

**The false claim: "tangency is a limit, so fillets are blocked."** It was checked
and it does not hold.
`Error::CannotFollowACrossingWhereItsSurfacesAreTangent` is constructed at exactly
**six production sites**: five on the marcher's quartic path
(`rcad-geom/src/march.rs:207`, `:242`, `:246`, `:264`, `:297`) and one torus row
(`along_a_crossing.rs:326`). `march` is a private module reached from **two
places**, `walk.rs:517` and `walk.rs:562`, both of which sit behind
`WhatARunLiesOn::QuarticPickedUpAt` (`walk.rs:197`, `:278`), which is built at
exactly one line (`:481`) called from exactly **two pairs**: sphere×cylinder
(`:422`) and cylinder×cylinder (`:450`). Every other pair in that match returns
`StraightLine`, a `Conic`, or an `Err` before any marching happens.
**A plane-and-cylinder pair never reaches it**, and
`intersect.rs:359-363` already answers the exactly-tangent plane-and-pipe case
with `EdgeRun::Straight` — the test is
`a_plane_resting_along_a_pipe_answers_with_one_endless_line`
(`the_degeneracies_a_crossing_has_to_answer.rs:117`). The tangent-plane refusal §43.6
files as a limit is a **plane resting on a torus's tube**, which is a different
shape from a fillet's tangency and is proved by a test of its own
(`along_a_crossing.rs:624-640`).

**The real reason to defer fillets** is that they need criterion 8 and a blend
operation, and neither exists, while the import rate has a fix with a working
precedent in the tree. That is a scheduling reason, not a limit, and it costs a
future unit two lanes rather than a research problem.

### 44.8 Criterion 11 stays open, and placing the builder does not close it

§43.8 item 11 asks for a decision between placing and baking.
**Decision: place it.** Write the `ShellUse` with the caller's isometry — ten
lines, exact by construction. Both blockers named in §43.8 are gone
(`HalfSpace::transformed` and `EdgeRun::transformed` in `0dc984d`; `EdgeView::run`
public and located in `2297970`), and baking costs measured exactness — a turned
picker loses exact squareness in 17 228 of 17 472 turns (§42.8) — to buy nothing
this unit needs. This reverses §42.8's choice, which was made when the blockers
were still there.

**And it must be said plainly that this does not discharge criterion 11.**
`add_extrusion` still writes `ShellUse { shell, loc: *placement, .. }` at
`rcad-topo/src/build.rs:571`, so §41.13's §19 violation is untouched. Placing the
builder makes it **consistent with** `add_extrusion`, not correct. Anyone reading
a green suite after this lane should read that sentence first.

### 44.9 What stays out, and why

| | why |
|---|---|
| **fillets** | §44.7 — a scheduling deferral with the price corrected, not a limit |
| **B-splines** | fails the consumer test: wants the free-form marcher §24.1 sent away. 68 files, and they wait |
| **healing the 76** | §44.6 — option A stands; C's price is corrected downward and the decision is not this unit's to make |
| **`Policy::tolerant` returning `Result`** | §43.8 item 3. 47 sites across four crates, collides with every other lane's write set, and its consumer is a hypothesis about callers rather than a named one (§13 rule 3) |
| **changing the seven `KnotVector` getters** | §43.8 item 2. The deliverable is a decisions.md entry with the argument attached and one test per getter showing its public predicate is sufficient — `contains`, `control_point_count`. This is `slice[i]` next to `len()`, and 110 call sites is not a lane's decision |

The four **cheap closures** that are in scope, all from §43.8: remove or deprecate
`Error::WhereALineMeetsATorusIsAQuartic` (`rcad-geom/src/error.rs`, constructed
nowhere in `src` — only two doc comments mention it); delete `rcad-topo`'s
`where_the_ray_meets` (`classify.rs:635`) in favour of
`rcad_geom::where_a_line_meets_a_surface`, which also closes
`Containment::CannotFireARayAtAFacesSurface` for a corner on the lattice; narrow
`StepFile`'s four methods that hand back unnameable types; and fix
`composite_wildcards.awk` to count uncovered variants rather than sites — noting
first, in writing, that `non_exhaustive_omitted_patterns` is the proper mechanism
and is unstable on the pinned 1.96.

### 44.10 How the work runs, and what green has to mean

**Lane 0 alone, then two waves.** At least three lanes at once, write sets named
before any lane starts, one deliverable per lane, stopping at about 45 minutes
(CLAUDE.md). **Every lane runs the `claim-audit` skill on its own deliverable
before reporting** — M5's lanes corrected each other four times while running and
its closing gate broke six more numbers none of them had caught. **One closing
gate re-derives every load-bearing number from the tree**, not from a lane's
report; a lane auditing itself is not a substitute.

`crates/rcad-topo/src/validate.rs` was wanted by four of five candidate lanes and
`error.rs` by three. The sequencing gives each exactly one owner at a time:

| stage | lane | owns |
|---|---|---|
| 0 | deserialize audit | `rcad-topo/src/{validate.rs,serialize.rs}`, `rcad-predicates/src/provenance.rs` |
| 1 | curved rounding | `rcad-topo/src/validate.rs`, `rcad-geom/src/{sphere,cylinder,cone,torus,lib}.rs` |
| 1 | interference | a new `rcad-topo` module, `rcad-topo/src/{error.rs,lib.rs}` |
| 1 | tangent direction | `rcad-topo/src/fuse/arrange.rs` |
| 1 | rate pinning | `rcad-step/tests/`, `justfile`, `.github/workflows/ci.yml` |
| 2 | two-step wires | `rcad-topo/src/{validate.rs,builder.rs,model.rs,error.rs,view.rs}` |
| 2 | the rest | as §44.4 lists |

**The one dependency to watch:** lane 0 and the curved-rounding lane both want
`validate.rs`. If lane 0's audit finds a third unguarded field it may want more of
that file, and wave one waits rather than merging around it.

**The curved-rounding lane's first act is a demonstration, not a number.** There
is **no curved analogue of `Plane::SIGNED_DISTANCE_ROUNDOFF`**
(`rcad-geom/src/plane.rs:77`), so the bound has to be derived and re-exported
before it can be spent. If a cylinder's representable-position error does not
decompose as cleanly as a plane's, **the item collapses and the lane reports
rather than inventing a number**. Whatever it publishes carries a §15.6 search
behind it, and it contradicts two doc comments that currently defer the curved
case — `validate.rs:1309-1313` and `rcad-step/src/refusal.rs:339-348` — both of
which go stale the day it lands.

**Verification. Green has to mean the corpus ran.**

- `mise exec -- just default` green, and **with `RCAD_STEP_CORPUS` unset the four
  corpus tests fail rather than skip**. Today all four print "skipping" and return
  green (`how_often_a_real_file_imports.rs:580`,
  `every_file_in_the_corpus_reads.rs:150`,
  `the_same_file_read_twice_by_two_readers.rs:281`,
  `a_real_file_written_out_and_read_back_is_the_same_solid.rs:433`), and nothing in
  `justfile` or `.github/workflows/ci.yml` sets the variable.
- **The 46 importing files are pinned by name, not by rate** — a rate hides which
  file moved — and the pinned list moves by the curved-rounding lane's own edit.
  `how_often_a_real_file_imports.rs:337` asserts only `!files.is_empty()` today;
  1 of 323 would pass it.
- `two_solids_described_at_the_same_place_are_still_called_a_valid_model`
  (`building_a_solid_out_of_faces.rs:358`) **fails** — criterion 8 landed. §19's
  shape.
- A `Model` deserialized with an infinite or negative `Approximation` magnitude is
  refused at the door, from bytes, and so is one carrying a non-finite vertex
  coordinate. The two fuzz targets assert it rather than walking past it.
- The `ModelData` asymmetry table is published, whatever it turns up.
- A tangent plane-and-cylinder pair fuses, or refuses for a reason that is true,
  checked against `degenerate_curved_fuses.rs:1539-1547`.
- `a_plate_bounded_by_an_arc_and_its_chord` builds and validates.
- The `validate.rs:436` skip's hit count over the existing corpora is published,
  whatever it is. Thousands makes it a correctness lane; zero means two doc
  comments disagree in the tree, and **that is the finding**.
- `just check-enums`, `just check-wildcards`, `just deny`, and the soak at
  `RCAD_SOAK_FACTOR=200`, all as M5 left them.

### 44.11 Corrections this section makes

Six, four of them to §42 and §43 and two to this unit's own planning. They are
here rather than as edits to the sections they correct, per §43.7's convention.

1. **Option C needs no new mechanism.** §42.9 and §43.8 item 12 both price it as
   *"a new `Cause` variant and a `Measures` arm"*, and
   `rcad-step/src/refusal.rs:347-348` repeats the first half in a doc comment —
   *"§42.9's option C, with a `Cause` of its own"*. `Cause::ImportedUncertainty`
   and `Measures::Length` both already exist, and the `Cause` is constructed
   nowhere in production. §44.6 has the demonstration. C is **one decision**, and
   that doc comment is a third place the corrected price has to reach.
2. **§42.9's D trigger was never scored, and §43.8 item 12 replaced D with C
   without saying so.** §44.6 scores it: not fired today, live after this unit's
   two lanes land.
3. **The two-step wire is bigger than §43.8 item 1 makes it sound.** It is **two
   independent length rules in two files** — `check_the_steps_close`
   (`rcad-topo/src/builder.rs:335-338`, raising `Error::ADraftWireIsTooShort`) and
   `check_wire_closes` (`validate.rs:304-309`, raising `Invalid::WireTooShort`,
   whose message says "at least three") — plus `winding_of` (`validate.rs:1347`,
   called from `check_flat_face`), which reads a wire's winding off its corner
   polygon so a face bounded by two half-circles sweeps exactly zero and cannot be
   told from a hole, plus `Model::add_wire_of_steps` (`model.rs:471`),
   `check_touched_wire` (`:560`), `wire_closes_on_one_edge` (`:587`) and
   `WireView::closes_on_one_edge` (`view.rs:690`). The reuse is already in the
   tree: `check_flat_face` skips one-edge wires at `validate.rs:928-931`, which is
   the same shape of exception a two-step wire needs.
4. **"The four curved surface files carry only a predicate name constant" is
   false.** Each also carries a private `SIDE_OF_*_ERROR_BOUND`
   (`sphere.rs:290`, `cylinder.rs:443`, `cone.rs:511`, `torus.rs:494`). The
   conclusion survives — those are the sign predicate's own filter bound, in the
   predicate's units, not a position error in a length, and none is re-exported —
   but the sentence as written understates what is already there, and the lane
   should start from those consts rather than from nothing.
5. **"The pinned count moves from 46 toward ~136" is an upper bound, not a
   projection, and the KiCad half of it is 134.** §43.5's 88 is drawn from the
   **161** KiCad files with a corner off a curved face, not from the **153** whose
   *first* refusal it was, so at most 88 files have their first blocker removed and
   each may then meet a second. The 2 NIST files in the 90 are in a separate family
   with its own count. **What the lane must report is the measured number and the
   filenames, not a target.**
6. **§43.5's KiCad refusal table does not sum, and its NIST column does.** The
   table says it counts each file once, named by what stopped it first. NIST:
   16 + 15 + 5 + 4 + 2 + 2 = **44**, exactly the family. KiCad:
   153 + 59 + 29 + 13 + 8 + 23 = **285** against 323 − 46 = **277** refused, over
   by **8** — the size of exactly one row, *a corner is not on the plane of the arc
   it ends on*. Either that row is not a first-refusal count or one row is eight
   too large. **This gate cannot re-derive it**: the corpus is not in the tree and
   `RCAD_STEP_CORPUS` is unset, which is §44.10's first verification line and is
   itself the reason the discrepancy survived a closing gate. **The rate-pinning
   lane settles it** when it publishes a per-file refusal reason, and §44.6's
   trigger is written as a range because of it.

### 44.12 What this gate does not know, and the prediction

- **Whether a cylinder's representable-position error decomposes like a plane's.**
  The whole curved-rounding item rests on it, and it is a demonstration nobody has
  run. §44.10 makes the lane's first act to settle it and its second to report if
  it does not.
- **What the `validate.rs:436` skip costs.** `error.rs:558-578` documents the fuse
  producing exactly the shape `holds.rs:400-402` abandons — a crossing cut at two
  corners leaving two arcs with the same two ends — while the only test of the
  check (`validate.rs:2013-2058`) has a doc saying *"no builder makes this and the
  fuse will not"*. One of those two sentences is wrong and instrumentation says
  which.
- **Whether interference has an answer that is not quadratic.** `Bounds::overlap`
  (`fuse/bake.rs:171-181`) is a box-level broad phase and `cuts.rs:78-82` is its
  current caller, so the shape exists; what nobody has costed is the narrow phase
  for the two cases roadmap §7.4 (`docs/roadmap.md:520-521`) names — fully
  contained, and touching but not overlapping.

**The prediction, recorded before the work and to be scored at this unit's closing
gate** (§30.2's cheaper half; §43.10's prediction is inherited alongside it).
§12.1's lesson — the money goes to code that was already written and looked
finished — is bet on for a third time: **the money goes to the two-step wire, not
to the curved rounding.** §43.8 item 1 reads as one rule and §44.11 item 3 already
found it is two rules in two files plus a winding question that has no answer for a
face bounded by two arcs; the curved rounding, by contrast, has a worked precedent
sitting in `validate.rs` that let 4147 of 4174 refused corners through. The second
guess is that **lane 0's audit finds a third unguarded field**, because it found
its second during planning and stopped only because planning ended. The third, and
the one this gate would most like to be wrong about, is that **criterion 8 goes
unstarted for a third unit running.**

