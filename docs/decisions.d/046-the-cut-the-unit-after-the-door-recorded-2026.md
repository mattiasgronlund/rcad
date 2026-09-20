## 46. The cut — the unit after the door, recorded 2026-09-03

*Written before any lane was dispatched, in §44's shape and not §42's. Every file
and line it cites was checked against the tree at `ef2b096`. It puts the
corrections first, because a lane that reads a stale price scopes itself wrong on
its first day.*

### 46.1 The name, and what the unit is

Called **the cut**. Roadmap §12 keeps M6 for local operations, per §44.1's
precedent and principle 6.

The name comes from §43.10's verdict on the kernel: *"a kernel that can build a
shape and cannot cut it is a modeller you can put things into and not take things
out of."* Cone and torus faces can be built, validated, measured, meshed and
classified. They cannot be fused with much, and the suite pins that as expected
behaviour — `how_often_a_fuse_with_a_cone_refuses.rs:316` asserts
`AFrustumAgainstABar` at nothing answered, and
`how_often_a_fuse_with_a_torus_refuses.rs:286` asserts *every* bend family there.

**The sentence the unit earns:**

> **A plane cuts a cone in a parabola, a hyperbola and a ruling, and the kernel
> can draw all three.**

The fuse clause — *and a frustum can be cut by a block* — is added **only if the
fuse lane lands**. "96 of 96" is a downstream count nobody has measured against a
fix, and §45.2 scored two numbers of exactly that shape at 88 → 50 and 75 → 1.

### 46.2 Planning re-priced the headline twice, both times downward

It looked like roadmap §13 row 1, the named schedule risk. It is not.

1. **It is not missing cone×cone SSI.** `intersect` already computes plane×cone
   and refuses two of the conic kinds that pair can produce.
2. **§43.6 had already written the consumer down** — `ACrossingThatIsAParabola` /
   `ACrossingThatIsAHyperbola`, *"consumer: a frustum against an upright block,
   96 of 96 refusals"*.

So the headline is a conic-classification gap inside a working function.

**The refusal is decided twice, independently, by the same polynomial.**
`E = (n·w)² − slope²·‖n×w‖²` is computed in `intersect.rs:1680-1704` for naming
and again through `Exact` in `walk.rs:1115-1122` for drawing. One
`match (apex_is_on_the_plane, leaning)` at `intersect.rs:1700-1712` has four arms
and answers one:

| arm | today |
|---|---|
| `(false, Pos)` | the ellipse — `NamedRun::the_whole_of(RightRoundWhereItsFacesCross)` |
| `(false, Zero)` | `Err(ACrossingThatIsAParabola)` — `intersect.rs:1706` |
| `(false, Neg)` | `Err(ACrossingThatIsAHyperbola)` — `intersect.rs:1707` |
| `(true, Zero)` | `Err(APlaneTouchingAConeAlongOneOfItsRulings)` — `intersect.rs:1711` |

**Naming needs no new `EdgeRun` variant.** `PartOfWhereItCrosses { and, picked_by }`
carries a parabola — it lies wholly in one nappe, so the half-space through the
apex square to the axis picks it — and each hyperbola branch, one per nappe. This
is fix (b) of the three `error.rs:335-348` already enumerates, and the one it
names shortest. It is also the only one that avoids adding a variant to a
`#[non_exhaustive]` enum that is `Serialize`/`Deserialize` and matched across two
crates.

**Drawing is the blocker, not naming.** `walk.rs`'s `Conic { centre, across, and,
size }` evaluates as `centre + across·cos θ + and·sin θ` (`walk.rs:787-801`). A
parabola has no centre and no semi-axes, so `where_a_plane_cuts_a_cone` cannot
return one and `walk` has no arm that consumes anything else. **The lane's real
cost is a second curve representation beside `Conic` and the walker arms that read
it** — and the lane demonstrates that choice rather than citing §43.6, for the
reason in §46.3.

### 46.3 Three corrections, before any lane reads them

**1. §43.6 and `error.rs:392-395` price the ruling wrongly.** Both say removing
`APlaneTouchingAConeAlongOneOfItsRulings` is *"one signature change in
`how_a_crossing_runs`"*. It is not one, and it is not a signature change.
`how_a_crossing_runs` (`arrange.rs:2914`) **already takes `run: &NamedRun`**. The
write sites are four, in three files: `intersect.rs:1711`, `walk.rs:1119`, the
cone arms of `a_plane_along_a_pipes_axis` (`arrange.rs:3057`, `Cylinder`-only and
`false` for every `Cone` pair) and `a_pair_that_crosses_in_a_conic`
(`arrange.rs:3023`) — and `assemble.rs:1967-1990`, whose doc says *"a cone never
reaches here running straight"* and which answering the ruling falsifies.

This is a price asserted and never checked, which is the failure mode CLAUDE.md's
rule about claims exists for. It matters because §43.6's endorsement of fix (b) in
§46.2 sits two lines from the claim that just failed, so it is not evidence.

**2. §45.10 understates the writer.** It says the writer has no `SURFACE_CURVE`,
so 52 of 97 importing files do not round-trip. The writer emits three things
(`write.rs:400-460`): a `LINE` for an open plane×plane edge, a `CIRCLE` for a
closed plane×cylinder edge whose plane is exactly square to the axis, and a `LINE`
for a cylinder seam. **Every other pair — plane×sphere, plane×cone, plane×torus,
cylinder×sphere, sphere×sphere — is `Err(ACurveTheFacesEitherSideOfItDoNotName)`.**
A ball that imports cleanly cannot be written out at all. Still deferred; now
described correctly.

**3. Two of the five surfaces are unreachable through the front door.**
`crates/rcad/src/lib.rs` re-exports `Plane`, `Sphere`, `Cylinder`, `Surface`,
`BSplineCurve` and `BSplineSurface` — but **not `Cone`, not `Torus`**, and not
`SIDE_OF_CONE`/`SIDE_OF_TORUS`. A facade caller can match `Surface::Cone(c)` and
cannot name `c`'s type, construct one, or build a cone face. This is on no
previous list — not §45.10, not §43.8, not §41.11 — and three closing gates
passed over it.

### 46.4 How the unit is scored

§44.2's change is kept: no a-priori criteria list, one closing gate, and a
verification list in which every line names something in the tree that changes
state.

Two rules the unit adopts from §45's findings:

- **The rate is reported and never targeted, and no file-count projection is
  published.** §45.2 demonstrated twice that a count of files a blocker stops
  *first* is an upper bound, not a projection. The 68 B-spline files are not
  priced here.
- **The gate's budget is reserved before the first lane starts.** §45.7 showed a
  lane's number can be true when taken and false when merged; §45.8 predicts that
  is how this unit loses too. A unit that spends its budget on lanes and gates on
  fumes gets exactly that.

**One verification line is worth naming here**, because it is the one that decides
whether the headline was right. `how_often_a_fuse_with_a_cone_refuses.rs:310`
asserts `refused > 0` for `AFrustumAgainstACornerUpCube |
APointedConeAgainstACornerUpCube`, and its own comment says *"re-derive the number
rather than relax it"*. Closing the hyperbola could take those families to none
refused and break it. **If it does not break, that is the finding**: the hyperbola
was not what refused them, and the fuse lane's price is wrong.

### 46.5 What the unit carries in, and what stays out

From §45.10, as opening lanes: the first-refusal count `brep.rs` has not
overwritten; `Invalid::Orphan` at the door; and the fuse's 30 production
`expect`s — unjudged for a third unit, and taken **before** the geometry lanes
because the cut has to touch `arrange.rs` and `assemble.rs`, which is where 29 of
the 30 live.

| out | why |
|---|---|
| **fillets** | §44.7's re-pricing is accepted — criterion 8 exists now, so it is one lane's machinery and not two — but a blend built on an operation that refuses its own primitives is built on sand |
| **another import-rate push** | cannot be honestly scored until the first-refusal count lands (§45.3) |
| **the writer** | §46.3 item 2. Deferred, described correctly |
| **option C, healing the 76** | one decision — *what is an imported corner's slop?* — but §45.3 means we cannot currently say which files are really in that bucket. Answered at the next gate, against the corrected table |
| **torus×curved, three-surface meets** | roadmap §13 row 1. Taking it in the same unit is how it eats the schedule |
| **`CannotWorkOutHowBigACrossingIs` at radius `1e-200`** | §43.8 item 4. A dynamic-range question; it belongs with the scaling work |

### 46.6 The decision rule for the first-refusal count, committed before the number is read

§45.2 and §45.4 both record numbers read after the fact and used three ways. The
rule is written first so there is nothing to read motivatedly:

- The curved-corner bucket **shrinks by more than half** once `brep.rs` stops
  overwriting → the published refusal tables were distorted, §45's second
  prediction scores as right, and option C is answered at the next gate against
  the corrected bucket.
- It **barely moves** → §45.3's relabelling was noise, and option D's trigger is
  scored on the spot: it fires or it does not, on the published number.
- **Either way**, no projection is published.

Both branches are survivable, which is the point.

### 46.7 The unit is renamed *the six arms*, and the sentence changes with it

§46.1 through §46.6 were written before the plan was grilled. The grilling read
the tree four more times and moved eight decisions; what follows corrects the
sections above, and where they disagree, these win.

**The name.** *The cut* is held for the unit where the conic-drawing half lands,
if it lands. This unit is **the six arms**, after the match it is named for. A
title is a claim, and §45.2's rule — a headline priced before a lane runs is
priced twice too high — applies to titles as much as to file counts. The unit's
guaranteed deliverable is in `rcad-geom`; the fuse clause sits behind a spike, a
lane and a merge.

**The sentence.** §46.1's is replaced by:

> **`where_a_plane_and_a_cone_cross` answers all six of its arms, and the kernel
> can draw every shape a plane cuts a cone in.**

Falsifiable by counting: open `intersect.rs:1702-1714` and see whether any arm
still returns `Err`. If the spike in §46.10 fails, the sentence keeps its drawing
clause for the two straight shapes and says plainly which two it lost and why.
The fuse clause — *and a frustum can be cut by a block* — is added only against
the assertion §46.9 pins, never against a printed number.

### 46.8 Six corrections, and the first of them is to a correction

Three of these correct §46.3, which was itself written as the corrections
section. That is worth saying out loud rather than filing quietly.

**1. §46.3 item 1 mis-priced the very mis-pricing it was correcting.** It said
the ruling's write sites are *"four, in three files"*. They are in **one file**:
the definition `arrange.rs:2914`, the callers `:1043` and `:4342`, the `HowItRuns`
enum `:452-464` and its `closes()` helper `:466-470`. §43.6 and
`error.rs:392-395` were wrong about the word (*"one signature change"* — it
already takes `run: &NamedRun`), and the correction was wrong about the scale, in
the same pass that produced three re-pricings the other way. A price nobody
checks stays wrong however many times it is rewritten.

**2. The match has six arms, not four, and it already answers two.** §46.2's
table lists four. `intersect.rs:1702-1714` also carries `(true, Pos)` →
`TouchAtOnePoint` at the apex, which is **answered**, and `(true, Neg)` →
`TwoLinesThroughAConesApexAreOneCurveThroughItself`, which is **refused and on no
previous list**. Two answered, four refused:

| arm | today |
|---|---|
| `(false, Pos)` | the ellipse — `the_whole_of(RightRoundWhereItsFacesCross)` |
| `(false, Zero)` | `Err(ACrossingThatIsAParabola)` |
| `(false, Neg)` | `Err(ACrossingThatIsAHyperbola)` |
| `(true, Pos)` | `TouchAtOnePoint` at the apex |
| `(true, Zero)` | `Err(APlaneTouchingAConeAlongOneOfItsRulings)` |
| `(true, Neg)` | `Err(TwoLinesThroughAConesApexAreOneCurveThroughItself)` |

**3. §45.10 understates the writer** — as §46.3 item 2 already said, kept here so
the corrected list is one list. Deferred, described correctly.

**4. `Cone` and `Torus` cannot be named through the front door, and five
`SIDE_OF_*` bounds for surfaces the kernel supports were never published.**
§46.3 item 3 found the first half. The second half is that of six `SIDE_OF_*`
constants — `cone.rs:18`, `torus.rs:15`, `plane.rs:15`, `sphere.rs:16`,
`cylinder.rs:18`, `half_space.rs:12` — `crates/rcad/src/lib.rs:357` re-exports
**one**. A facade caller cannot name the bound its own results were certified
against for any surface but a half-space.

**5. `brep.rs` overwrites the real refusal twice, and a published column measures
the wrong corner.** `brep.rs:600-626` substitutes
`a_corner_provably_off_a_curved_face` (`:643-675`, which takes nothing derived
from `why` and rescans every non-planar face), and when that finds nothing falls
through to `a_corner_provably_off_the_plane_of_its_own_arc` (`:698`), whose own
rustdoc at `:680-697` calls it a rounding artefact of how the plane is stored.
The corpus report's `miss={...} spacings` column
(`how_often_a_real_file_imports.rs:409-419, :688`) is computed from the
**substituted** refusal — a measurement of a corner that may have had nothing to
do with the failure. §45.3 said the refusal name is not the reason; this says the
number beside the name is not the reason's either.

**6. `Invalid::Orphan` already exists** — `error.rs:950`, raised in five places
in the full sweep (`validate.rs:179, :291, :318, :431, :578`). What is missing is
a check at the deserialize door, not a variant. Worth a correction because a lane
that did not know would have budgeted for one.

### 46.9 The headline family was unasserted, and is pinned before any lane runs

`how_often_a_fuse_with_a_cone_refuses.rs` asserted every family in its table
except the one the unit rests on. `AFrustumAgainstABlock` was *"reported, not
judged"*: a lane could have taken it from 0 answered to 96 and no test would have
noticed.

It now carries `assert_eq!(answered, 0)`, written by the dispatcher **before the
first lane was briefed**, so nobody can be said to have written their own
scorecard. Measured at the pin: **96 fuses, 0 answered, 96 refused, and all 96
give one reason** — *"a plane cuts this cone in a hyperbola"*. The family is
hyperbola-blocked outright, not partly, which prices the fuse work more sharply
than §46.2 could: what stands between this family and an answer is the drawing
lane, and the arrangement arms are behind it rather than beside it.

### 46.10 The spike, and what a fail re-scopes

Four explorations removed every structural risk the plan feared but one.
`what_a_run_between_these_two_lies_on` (`walk.rs:362-366`) is **already
run-keyed**; one pair yielding two differently-shaped runs is a working, tested
pattern (`intersect.rs:1836-1847` mints two runs differing only in `picked_by`,
and `walk.rs:1332-1348` filters by the run's ends and pickers and demands exactly
one survivor); and three of the four refused shapes need **no new geometry at
all**.

What no precedent covers is **endlessness**. A parabola arrives with
`nowhere_in_particular` ends both sides (`meet.rs:130-133`) and `walk` produces
points *between* the ends — of which there are none. `error.rs:343-346` already
raises this as the objection to fix (c).

So: a **20-minute spike**, its rule written before it runs. Target a right-angled
cone and a plane with normal `(0,1,1)` — the `E = 0` case
`the_seven_shapes_a_plane_cuts_a_cone_in.rs:1-29` says is reached exactly.
**Pass:** `walk` returns at least two points from a run with
`nowhere_in_particular` ends, every point on both surfaces within the published
roundoff. **Fail:** twenty minutes gone without that. *"It looks tractable"* is a
fail. *"The representation works but the range is open"* is a fail.

On fail the drawing lane drops to naming-only, the sentence loses its drawing
clause for parabola and hyperbola and keeps it for the two straight shapes, and
the recorded finding is that **endlessness, not conic classification, is what the
record has been mis-pricing since §43.6**. The spike runs beside the first wave,
which is dispatched without the lanes that depend on it: the spike can re-scope a
lane, not the unit.

### 46.11 The budget, in agent-hours, with the gate's share reserved first

§46.4 said the gate's budget is reserved before the first lane starts. Here is
the number: **9 agent-hours total, of which the gate takes 1.5, reserved first.**
The spike takes 0.35. What is left buys lanes at ~0.75 each.

If wave 2 runs short, the drop order is **the fuse lane > the quarter arc > the
two half-circles**: the fuse lane is the headline, the quarter arc pairs with the
front-door lane's `--ignored` fix, and the half-circles lane changes no published
sentence. Every lane commits at its halfway point whether or not its deliverable
is done, and every report carries a *what is left* section, empty if it finished
— lanes died mid-work last unit and their work died with them.

### 46.12 Three findings this unit hands the gate rather than fixing

Named here so the gate does not have to rediscover them, and so they are on a
list if the unit runs out of budget.

1. **The compiler forces zero edits when the run vocabulary grows.** `EdgeRun`
   and `SurfaceMeet` are `#[non_exhaustive]` in `rcad-geom` (`meet.rs:28-55`,
   `:171-173`), so every `match` on them in `rcad-topo` carries a wildcard and can
   never be exhaustive. `a_sixth_kind_of_surface.rs:9-18` already records this for
   `Surface`; nothing records it for runs, and a new run reads as *not straight*
   and *not closed* in a dozen places without a compiler error.
2. **22 production `panic!`/`unreachable!` in the fuse have gone uncounted for
   three units**, alongside the 30 `expect`s that have been counted.
3. **`TwoLinesThroughAConesApexAreOneCurveThroughItself` asserts a curve the
   kernel does not have to build.** Two straight runs told apart by pickers is
   enough, and `two_lines_told_apart` (`intersect.rs:1578-1608`) already does
   exactly that for another pair. The name argues for a representation the fix
   does not need.

### 46.13 The decision rule was written twice and the two forms disagree

§46.6 says the bucket **"shrinks by more than half"**. The unit's own plan, in
the dispatcher's notes, says **"≥ 50%"**. Nobody noticed until a lane came back
with a number that lands exactly between them: the KiCad bucket goes **168 → 84**,
which is 50.0% — over the line by one reading and short of it by the other.

**§46.6's text wins**, and the reason is written here rather than discovered
later: it is the form that is committed to the repository, and it is the
*stricter* of the two, so choosing it is choosing against the branch that
flatters the unit. A rule read after the number is a rule that was not a rule.

This is not the score. The gate generates the table at the final tree, over
**both** corpora and against the denominator §46.6 names — files that reach
`SolidBuilder::build`, now counted and printed by the lane as
`reached_the_solid_builder`. What is fixed here is only which side of the line
50.0% falls on if the final number lands there again.

**And a lane's number is not the unit's.** A later lane in this same unit fixes
the mesher bound that makes a hand-built fixture start importing, which moves
these rates — §45.7's failure mode, which is why no lane in this unit publishes
a rate and the gate publishes all of them.

