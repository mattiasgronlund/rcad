## 15. Shared meanings, settled before M1 and M2.5 fan out

*Recorded 2026-08-30, before either milestone starts.*

§13 rule 2 says a milestone is shaped gate, fan, gate. §13 rule 4 says the
shared meanings get settled in the gate, in writing, because two people making
the same decision independently make it twice, differently — §12.4 is what that
looks like afterwards.

M1 and M2.5 are going to run side by side (§15.1). That makes one gate serve two
milestones, so this section is it. Everything here is a meaning that both would
otherwise have to decide, and every one of them is decided here rather than in
either milestone.

### 15.1 M1 and M2.5 do not depend on each other, and M2.5 does not need M2

`rcad-topo` depends on `rcad-geom` and never the other way round (§7.2). M1 is
work inside `rcad-geom`; the M2.5 fuse is work inside `rcad-topo`. Neither
consumes anything the other produces:

- The planar fuse needs `Plane` — including `basis`, `plane_coordinates_of`,
  `point_at` and `meet_of_three` — plus `Segment`, `Point`, `Isometry` and
  `Surface::as_plane`. All of that shipped in M0. Nothing in a knot vector or a
  control net reaches it. Edges do not even store a curve (§3.1), so M1's
  `Curve` never appears in the fuse's path.
- M1 consumes nothing from topology. That is the whole point of §7.2, and it is
  what makes "ship M1 standalone" possible at all.

**M2.5 also does not need M2.** M2 is tessellation and rendering on curved
geometry, and curved geometry is M1's output. Building a planar solid and
validating it landed in M0 instead (§3.1), and classification already
triangulates faces to cast rays through. So the fuse's inputs are all present.

This is recorded as a departure in the §1 sense: the numbering says M1, M2, M2.5
in that order and the dependencies do not. Running M2.5 alongside M1 serves
§1.7's own argument, which is that the refusal rate is worth having early — it
arrives earlier still. A later reader who finds the two milestones landing
together should read this, not the numbering.

**What that costs, and the honest counter-argument.** §13 rule 6: review is the
constraint, not authoring, and two milestones in flight is more code per unit of
review. §12.1 is the evidence — three bugs shipped through a green 335-test
suite and were found only by review. Rule 6 permits going wider across a crate
boundary that is already a stable contract, and this one is, for planar work.
Permits, not recommends. If review starts slipping, the milestone to pause is
M2.5, because M1 is on the critical path to M2, M3 and everything after.

### 15.2 An `Approximation` magnitude is a length

**Settled: the magnitude on an entity is how far a point could be out of place,
in the model's own unit. Never the raw value of a predicate.**

§12.4 fixed the budget end of this and left the tagging end alone; §12.6's
second gap is the leftover. Today `resolve_under_policy` is handed the
`Measures` of the predicate that gave up, uses it to pick a budget, and then
stores the raw enclosure in the tag and throws the units away. So an
`Approximation` off `orient2d` is an area, off `orient3d` a volume, and off the
dynamic-range check a length — three different quantities in one `f64` field
with one name.

The conversion is already written down, in `Measures`' own doc comment: move a
corner by `w` at working size `s` and a predicate of that degree moves by about
`w · s^(degree−1)`. Turn it round:

```
wobble = enclosure ÷ working_size^(degree − 1)
```

Length divides by one and passes through unchanged, which is why the
dynamic-range check was already right. Area divides by `s`, volume by `s²`.

**What was already right, and what this adds.** Since
`max_slop = 64·ε·s^degree`, the wobble a tolerant policy permits already comes
to `64·ε·s` whatever the predicate degree — §12.4 got that far when it made the
budget carry units. What it did not do is put the *tag* in those terms, so the
number stored on an entity is still in the units of whichever predicate happened
to produce it. This rule finishes the job: budget and tag measure the same
thing, which is what lets §15.3 compare them and §5.4 order them.

The uniform `64·ε·s` is worth a test either way, because it is the property that
breaks first if a degree is ever added to `Measures` without its conversion.

**Which working size.** The one the tolerant policy already carries, since the
conversion happens inside `resolve_under_policy`'s tolerant branch and nothing
outside it produces a tag from an enclosure. No new plumbing. That the model's
`Scale::nominal_size` is a second copy of the same number is a seam worth
watching, not one to fix here.

*Amended 2026-10-09 (Q1467): the seam opened, and is closed.* The fuse's
judgment (§17.2) read its whole log at the model's `Scale::nominal_size`, while
`resolve_under_policy` had accepted each record at the policy's working size.
Under a tolerant policy bigger than the model, an area or a volume came out
bigger at the judgment than when it was accepted, by the ratio of the two sizes
to the power of one or two, and the fuse refused a solid built only out of
decisions the policy had accepted (§17.8 names the pair). `Model::fused_under`
now reads the log at the policy's working size. It reads it at the model's only
under `Strictness::Certified`, which carries no size of its own, and whose
budget of zero gives the same verdict at any positive size unless a record's
wobble rounds down to zero. Nothing else in the crates' sources reads a record
at the model's size.

**This is an accounting figure, not a bound, and that has to be said out loud.**
The true sensitivity of a predicate to a moved corner is its gradient at the
configuration in hand, and a sliver triangle has a small one — the same
enclosure there means a larger real wobble than `s^(degree−1)` suggests. What
the rule buys is a number that can be compared between two entities and summed
across an operation, which is the only thing a stored magnitude is for.

Rejected: use the actual gradient. More faithful, and it gives up the one
property that makes the number worth storing — the tag would depend on where the
part sits and what shape the triangle is, so two tags could no longer be
compared, and §5.4's ordering check would be comparing noise. It also costs a
gradient evaluation on every non-certified decision. Noted as the escape hatch
if the stand-in ever proves too loose to act on.

**The named consumer**, per §13 rule 3, already exists:
`check_approximation_ordering` in `validate.rs` compares vertex against edge
against face, and §5.4's `tol(vertex) ≥ tol(edge) ≥ tol(face)` is only a
statement about anything once all three are lengths.

### 15.3 An operation judges a wobble, and the field is named for it

`Policy::max_enclosure` and `enclosure_is_acceptable` still have no caller
(§12.6, first gap); §1.7's exit criterion 7 gives them one. Before that caller
is written, two things get settled.

**The operation judges in lengths.** Once §15.2 makes every tag a wobble, the
operation-level question — is this solid's worst slop small enough to hand
back — is a question about a length. So `max_enclosure`'s `Measures` argument is
not a choice the caller gets to make; there is exactly one right answer to pass,
and an argument with one right answer is a way to get it wrong.

**So the names change**, while there is still no caller to break:

| Was | Is | Means |
|---|---|---|
| `Policy::max_enclosure(Measures)` | `Policy::max_wobble()` | the widest a point may be out of place, `64·ε·working_size` |
| `Policy::enclosure_is_acceptable(f64, Measures)` | `Policy::wobble_is_acceptable(f64)` | judged against the above |
| `ProvenanceLog::worst_enclosure()` | `ProvenanceLog::worst_wobble(working_size)` | the worst wobble in the log |
| `Policy::max_slop(Measures)` | *gone* | see below |

**Amended 2026-08-30, while implementing.** `max_slop` is removed rather than
kept alongside. It was the same budget expressed in a predicate's own units, and
`resolve_under_policy` was its only caller. Leaving it would mean the predicate
judging an enclosure against one number while the operation judges a wobble
against another — equal in exact arithmetic, not equal in `f64`, so a decision
could be accepted at the predicate and then push a solid over the operation's
budget by an ulp. Instead the conversion happens once, before the judgement, and
the number that is judged is the number that gets stored. `Measures::wobble_of`
carries the units knowledge and has a real caller in `Record::wobble`, so §13
rule 3 is satisfied where it was not before.

`Record::enclosure()` stays as it is and keeps returning the raw value in the
predicate's own units — it is the honest record of what the cascade saw, and
that is what a record is for. `Record` grows a `measures()` so the units are no
longer lost at the moment the record is made.

`ProvenanceLog::worst_enclosure` is the one that has to go rather than merely
being renamed. It folds a maximum over records whose values need not share a
degree. No operation in the tree mixes degrees into one log yet — the profile
builder records only `orient2d` areas, classification only `orient3d` volumes,
`Model::validate` only the dynamic-range check's length, and the swept-area
check builds a fresh log inside the error it returns. Nothing stops the mixing
though: a caller may hand the same log to two operations, and `drain_into`
exists precisely so one operation's log can be folded into another's. The fuse
is the first thing that will mix degrees on purpose. That is §12.4 one level up,
waiting for a caller.

§2.6 lists these as fields of `Policy` and it still does. This renames them, and
records that the old names were the invitation: a field called `max_enclosure`
asks to be compared against a raw enclosure, which is the units bug it exists to
prevent.

### 15.4 `Decision` does not grow during the parallel window

§3.6 says `Decision` is `#[non_exhaustive]` because it grows at every milestone,
and §2.1 says every call site matches all its cases on purpose. Both are still
true. Together they mean a new variant landing in one worktree breaks every
match in the other — and the matches that do not break are worse, because
`classify.rs` and `validate.rs` handle an unknown variant by panicking, which
turns a merge into a runtime failure somewhere else.

**Neither M1 nor M2.5 adds a variant.** Nothing either needs one for: a NURBS
point that cannot be placed is `Undecidable`, which is what `Undecidable` is.
If one of them turns out to need one anyway, that is not a change to make inside
a fan piece — it stops the fan, reopens this gate, and gets written down here.

### 15.5 No decision is taken on projected coordinates

**Settled: the fuse decides in 3D, on stored coordinates, through `orient3d`.
`orient2d` keeps taking exact `[f64; 2]` and nothing else.**

Projecting into a face's plane is tempting, because splitting a planar face is a
2D problem and `plane_coordinates_of` is right there. It is also a way to throw
away exactness for free: `Plane::basis` normalizes, so projecting a vertex whose
3D coordinates are exact produces 2D coordinates that are not. Every projected
point is a constructed point, level 3 of the cascade can no longer run on it
(§2.5), and the fuse would be manufacturing the undecidability §1.7 exists to
measure. Exit criterion 8 would then report a number about our projection, not
about the geometry — §14.7's warning, arriving by a different road.

`orient2d`'s only caller today is the profile builder, whose input is `[f64; 2]`
from the caller and genuinely exact. That stays true.

If the fuse turns out to need a 2D predicate on constructed input, growing
`orient2d` to take a point that may be exact or constructed is gate work, not
fan work: it is a change to `rcad-predicates`, which §15.9 freezes.

### 15.6 A bound lives beside the expression it bounds; a safety factor lives beside its consumer

§12.6's fourth gap is `Model::validate`'s `allowed_wobble`, whose `64·ε` is a
guess where a rigorous `3·ε` exists. It is half fixed: `MEASURING_ROUNDOFF` now
derives the `3·ε` in its doc comment and says plainly that the twentyfold on top
is not derived. What it does not do is live in the same crate as
`Plane::signed_distance_to`, the expression it is a bound on.

M1 is about to write a great many bounds — partition of unity over p+1 terms,
evaluation, derivatives — so the rule matters more than the one case:

1. **A bound on an expression is a named public constant in the same module as
   the expression**, with its derivation in its doc comment. `ORIENT3D_ERROR_BOUND`
   is the pattern.
2. **A safety factor is a separate named constant at the consumer**, with its
   own reason. Multiplying a derived bound by twenty is a judgement about how
   much slack a caller wants, and it is not part of the bound.

Applied to the open case: the bound moves to `rcad-geom` next to
`signed_distance_to`; the slack factor stays in `validate.rs` under its own
name. This is why the fix lands before the fork — it touches `rcad-geom`, which
M1 owns from then on.

**Amended 2026-08-30, while implementing. The `3·ε` was not a bound.** §12.6's
fourth gap, and the paragraph above as first written, both called `3·ε` the
rigorous figure that `64·ε` was a guess at. Writing the derivation out properly
showed it is not: it counts half an ulp per *term*, but a rounding costs half an
ulp of the **running sum**, which is larger than any one term. Charging each
term for every operation it passes through, and using `|n₁| + |n₂| + |n₃| ≤ √3`
for a unit normal, gives `u·(4√3 + 1)·L` — just under `4·ε·L`.

So `Plane::SIGNED_DISTANCE_ROUNDOFF` is `4 * f64::EPSILON`, and the slack at the
consumer is 16 rather than the "little over twenty" the old comment described.
The product is `64·ε` either way, so nothing the kernel does changes; what
changes is that the figure is now derived rather than asserted.

Worth noting how it was caught, because it is §14.1 question 1 exactly: the
number had a derivation written next to it and was still wrong, because nothing
executed the derivation. The bound only became true when a test evaluated the
same expression in exact arithmetic and subtracted. **A published bound needs a
test that could fail, not a paragraph.** That test cannot show a bound is
*tight* — rounding errors cancel, and it passes at `2·ε` and `3·ε` too — so the
derivation is still what makes it a bound. The test is what stops the expression
drifting out from under it.

### 15.7 The intersection result type lives in `rcad-topo`

§1.7 requires the fuse to consume an intersection *result* that carries an
enclosure and an `Approximation` from its first line, so that M3's curved
intersector is a new producer rather than a new contract.

**It goes in `rcad-topo`.** §13 rule 3: its only consumer is the fuse, and the
fuse is there. Putting it in `rcad-geom` where M3's producer will eventually
live builds a thing with no consumer in the milestone that builds it — the
mistake §12.6's first gap records — and it would put the one type the two
milestones share into the crate M1 is rewriting.

Its `Approximation` is a length, per §15.2. The planar producer leaves it at
zero and fills in nothing else; that it is never populated in M2.5 is the point.

When M3 gives it a second producer, moving it down to `rcad-geom` is a
relocation and nothing more, since `rcad-topo` already depends on `rcad-geom`.
Recorded so that move reads as planned rather than as a layering mistake being
corrected.

**Amended 2026-08-30, while planning M2.5.** "Fills in nothing else" is half
wrong, and as written it would have someone wire the endpoint enclosure to
zero. The `Approximation` — how far the computed curve strays from the true
one — really is zero for two planes and stays zero forever: the meet of two
planes is a straight line and the plane pair *is* that line, with nothing
approximated about it. M3's marcher is its first producer, exactly as this
section says. **The endpoint enclosure is a different field and it is not zero
and never will be**, because `Plane::meet_of_three` solves by Cramer's rule and
Cramer divides. Every end of a planar face meet is therefore a constructed
point with a real box around it — §2.5's case, which §17.1 turns from the
exotic path into the only path. So the planar producer populates the enclosure
on every meet it returns, and it is only the fidelity that stays at zero.

### 15.8 "Refusal rate" and "deserved refusal" are two different numbers

§1.7's exit criterion 8 wants a refusal rate. §14.7 wants an acceptance oracle —
proof that a refusal was deserved rather than a timid filter. They are the same
worry from two sides, and left alone the two milestones would produce two
numbers with one name.

- **Refusal rate (M2.5, criterion 8)** is counted **per operation**: the share
  of fuse operations over the §1.7 corpus that come back `Error::Undecidable`
  under `Policy::certified()`. Per operation, because that is the unit a user
  experiences a refusal in, and because §1.7's risk is about whether the kernel
  is usable.
- **The acceptance oracle (M1, §14.7)** is **per case, pass or fail, not a
  rate**: cases whose answer is exactly computable, which `Policy::certified()`
  must decide. A single one failing is a bug, not a percentage going up.

Both get written down; neither gets called "the refusal rate" except the first.

### 15.9 What lands before the fork, and who owns what after

**Before either worktree opens.** Each of these needs a test that fails without
it, per `CLAUDE.md`:

1. `Record` carries its `Measures` and can report a wobble (§15.2, §15.3).
2. `resolve_under_policy` tags a wobble rather than a raw enclosure (§15.2).
   The test worth having is the falls-out property: one tolerant policy permits
   `64·ε·working_size` of wobble whatever the predicate degree.
3. `ProvenanceLog::worst_enclosure` becomes `worst_wobble(working_size)`; the
   `Policy` pair is renamed (§15.3).
4. The `3·ε` bound moves next to `Plane::signed_distance_to`, and the safety
   factor stays at its consumer under its own name (§15.6).

Items 1–3 are in `rcad-predicates` and item 4 is in `rcad-geom` — both crates
the fork would otherwise contend for, which is why all four land first. None is
large. Doing them afterwards means doing them twice.

**All four landed 2026-08-30.** Two things came out of the work that the plan
above could not have known, both recorded where they belong: `Policy::max_slop`
is removed rather than kept (§15.3), and `3·ε` was never a bound (§15.6). The
change is covered by four tests that fail without it — three on what a tag holds
and how a log is summed, one on the plane bound — and the suite is 404 green.

**After the fork.**

| Crate | Owner during the window | Rule |
|---|---|---|
| `rcad-geom` | M1 | M2.5 may **add** items — a two-plane meet beside `meet_of_three` is the expected one. It may not change what anything already there means. |
| `rcad-topo` | M2.5 | M1 has no business here. |
| `rcad-predicates` | nobody | Frozen. A change reopens this gate and stops both fans. |
| `rcad` facade | both | Re-export lists only. A textual conflict, never a semantic one. |
| `docs/decisions.md` | both | Append only. M1 writes §16, M2.5 writes §17; those numbers are reserved now so the two never collide. |

Two worktrees, and each gets its own `target/`. Sharing `CARGO_TARGET_DIR`
between them only makes the builds wait on each other's lock.

§13 rule 5 is satisfied without anyone arranging it: M1 generates knot vectors
and control nets, M2.5 generates extruded polygons and the §7.4 catalogue. The
two suites cannot go green together for the same wrong reason.

**Amended 2026-08-30, while planning M2.5.** The table's `rcad-geom` row
predicts "a two-plane meet beside `meet_of_three`". **That is a deliberate
non-addition: M2.5 will not write one.** The plane *pair* is the line already,
exactly and for free — `{A, B}` names it with no arithmetic at all — and every
question the splitter actually asks is a meet of three planes, which exists.
Turning the pair into a point plus a direction would normalize a cross product
and divide, replacing an exact object with a rounded one before anything used
it, and that is §15.5's mistake arriving by a different road: the fuse
manufacturing the undecidability §1.7 exists to measure.

What `rcad-geom` gets instead is one added predicate,
`Plane::agreement_with`, which answers whether two planes describe the same
plane by exact proportionality of their coefficient 4-vectors, and which way
each faces (§17.4). And `rcad-predicates`, which this table freezes, gets
exactly one additive change: `Cause::ConstructedPoint` (§17.2). It is taken
*before* M1's worktree opens rather than during the window, so the freeze in
the table is real rather than aspirational — the row means nobody may change
that crate while two fans are running, and running M2.5 alone until the change
has landed costs a day and satisfies it literally.

### 15.10 A rate that moves is reported with the cases that moved it

*Added by §48.8, and standing from there on. Every later unit inherits it.*

**When a published number changes, name the cases that changed and say what they
have in common.** A count is not evidence for the sentence it is offered under.

Three consecutive units lost to the same gap and each thought it had closed it.
§45.7: a lane's number was true when taken and false when merged. §46.9: the
family the unit rested on was reported and never asserted, so a lane could have
moved it with no test noticing. §47.2: the number *was* pinned, it broke in the
right direction, it was re-derived — and the sentence written beside it was
still false, because nobody asked **which** cases had started answering. All six
were the one row where the two solids miss each other entirely.

So the rule is not "pin the number". It is that a number moving in the right
direction says nothing about whether the thing the number stands for happened.

**The closing gate may not accept a moved number without its case list.** A unit
that cannot say what its moved cases have in common has not finished measuring.

---

