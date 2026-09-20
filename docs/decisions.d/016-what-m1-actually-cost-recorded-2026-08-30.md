## 16. What M1 actually cost, recorded 2026-08-30

*Written by M1, per §15.9's reservation of this number.*

`rcad-geom` can now represent, evaluate and subdivide NURBS curves and surfaces,
and can hand a NURBS point to the predicate cascade **carrying an enclosure** —
the seam §13 rule 2 named as M1's closing gate. The suite went from 404 to 549.

`rcad-predicates` was not touched, so §15.9's freeze held. No `Decision` variant
was added, so §15.4 held. §12.6's fifth gap is closed: `Interval` is re-exported
from the facade, because callers now take `PredicatePoint::enclosure()` apart and
a type the rustdoc names has to be one the facade exports.

### 16.1 The schedule risk was named twice and landed in neither place

§12.1's lesson was that the risk is not the part that looks hard, so the M1 plan
named two candidates: the enclosure width, and — because it is textbook and
finished-looking — the de Boor evaluator. Neither failed.

On shapes of a stated working size the enclosure came in at **2 to 20 `f64`
steps of that size**, flat across working sizes from 1 to a million and across
degrees 1 to 5.

| | curve, well conditioned | curve, wide range | patch, well conditioned | patch, wide range |
|---|---|---|---|---|
| steps of the working size | 2–11 | 2.5–14 | 5–14 | 4–20 |

**The answer for M3 is conditional, and the condition is the interesting half.**
For well-conditioned input this settles it: tens of steps, not thousands, and
certified marching works. But the first version of this figure was measured with
every knot vector drawn from the same six small whole numbers — the monoculture
§16.9 records — and once the generators put a span of a million steps next to a
span of one, the tail moved. Over a million draws of the six-decades-of-weight
family: **median 15 steps, 99.9th percentile 986, worst 66,120.** That is a
parameter landing a hair below a knot in a span a million times wider than its
neighbour, which is the curve being ill-conditioned rather than the arithmetic
being sloppy — the box is honest about it, which is the whole point of having
one. **M3 will need centred forms for that case**, and it now knows which case,
eighteen months before it would otherwise have found out.

That family is therefore held to a scale-free ceiling — the enclosure's
half-width against the published `f64` bound, worst measured 0.145 — because a
ceiling in steps of the working size would have to sit near a million to stop
flickering, and a ceiling that loose asserts nothing.

Where the cost actually went was neither algorithm. It went to **defining the
numbers before measuring them** — §16.2 and §16.3 below, and the ceiling above.
All three are the same mistake in different clothes: a quantity that is
meaningless until you say what it is measured against.

### 16.2 An enclosure width has to be measured against the right thing

Reported in steps *of the coordinate*, degree-5 patches came back at 2·10⁷ and
looked exactly like the schedule risk landing. They were not. The worst case is a
coordinate of −1.2·10⁻⁷ with an absolute width of 5.6·10⁻¹⁶, which is five steps
of the control net that produced it. Dividing a scale-free width by a coordinate
that crosses zero says nothing about the arithmetic and everything about where
the curve happens to pass near an axis.

This is the same trap `EVALUATION_ROUNDOFF`'s own derivation opens with, and it
was walked into anyway, one file apart. The bound is stated in terms of the
largest control coordinate the evaluation touches *because* nothing relative to
the answer survives a de Boor result near zero — and then the width was reported
relative to the answer.

Both readings are now reported. The ceiling is on the second, and it is
machine-checked in the normal suite rather than read off a `criterion` run,
because §12.5 was honest that nobody notices one of those regressing.

### 16.3 A bound's operation count is not its rounding count

`EVALUATION_ROUNDOFF` charges **nine roundings per de Boor stage.** Counting the
operations gives seven, and seven is wrong. Two of the roundings form the
coefficient `a`, and their error is not carried down one chain — it is charged at
full size against the running values, because `|δa|·|d[j]|` does not shrink when
`a` is small.

§15.6 said a published bound needs a test that could fail, not a paragraph, and
M1 is the second run of that experiment. The test — the same recursion in exact
rational arithmetic, subtracted — is in the tree and it works. It is also **not
sufficient, measurably**: of about fifteen deliberate mutations of the evaluator,
every algorithmic one was caught, and one survived. Dropping the `w'` term from
the rational divide is worth about a factor of two, and the exact-arithmetic test
carries roughly 300× slack, so it only fails below about a five-hundredth of the
published constant.

That is not a defect in the test; it is §15.6's own claim arriving with a number
on it. **The derivation is what makes it a bound. The test is what stops the
expression drifting out from under the derivation.** Neither substitutes for the
other, and now we know the size of the gap between them.

### 16.4 A frozen crate is a design constraint, not only a process one

The M1 plan claimed no change to `rcad-predicates` was required, because
`Point::Constructed` and `Interval` are already public. That was very nearly
right and missed one thing: **`Interval` has no public division.** Interval de
Boor divides — at every knot difference and at the perspective divide — so the
enclosure could not be built from the frozen API as it stood.

It was built locally in `rcad-geom` instead, from `Interval::new`, `lo` and `hi`:
IEEE division rounds to nearest, so the true quotient lies between `q.next_down()`
and `q.next_up()`, and the four corner quotients widened by a step each way
enclose. It costs a couple of steps of width over a directed-rounding divide.

Recorded because the outcome was better than the shortcut would have been — the
soundness argument is local, written down and tested — but the freeze was **not
free**, and a future gate should not assume a frozen crate's public surface is
sufficient just because the types are public. Check the operations.

When `rcad-predicates` next opens, moving the divide down to `Interval` is a
relocation and nothing more.

### 16.5 The acceptance oracle could not be built first, and that is worth saying

§14.7 says build it first, not last, and §15.8 says it is per case, pass or fail,
never a rate. Both held. "First" did not, and could not have.

The oracle's subject is the *enclosure*, and the enclosure is the last thing M1
builds. What was built first is its other half: the exhaustive check that
`KnotVector::new` accepts exactly what an independent transcription of the six
rules accepts — every non-decreasing sequence over the whole numbers 0 to 7 for
degrees 1 to 3, **182,595 of them, enumerated rather than sampled**, zero wrongly
refused and zero wrongly accepted.

So §14.7's instruction resolves into two: the *acceptance half of every
constructor* goes first and can, and the *acceptance half of the cascade* goes
with the thing it judges. Only the second was what §14.7 had in mind, and only
the first is schedulable early. A milestone that reads §14.7 as one instruction
will either build it late or build the wrong one.

What it bought, when it did land: the quarter circle with weights `(1, 1, 2)` is
the unit circle exactly and reaches `(3/5, 4/5)` at `u = 1/2`, which is not an
`f64`. Handed to `Sphere::side_of` as an exact triple, that point is **certified
`Pos`** — off a circle it is exactly on. Handed over as a box it is
`Undecidable`. Neither assertion needs a tolerance, and the contrast is the whole
argument for the seam.

The acceptance half is asserted too, and it is the half §14.7 exists for: the
same points *are* certified against spheres of half and twice the radius, at both
clamped ends, on the extruded pipe and on a flat patch's plane. A timid filter
passes every other oracle on the list by refusing everything.

The quarter-sphere patch the plan called a stretch goal worked: the dyadic
weights do survive the revolution, giving `1,1,2,1,1,2,2,2,4`, and the octant
point at `u = v = ½` is `(9/25, 12/25, 4/5)`, whose squares sum to exactly one.

### 16.6 Rule 3 removes exactly the pieces rule 2 promised would fan

§13 rule 2 says a milestone is shaped gate, fan, gate, and named M1's algorithms
as the fan. §13 rule 3 says a part with no named consumer is deferred. Applied to
roadmap §4, rule 3 cut about half of it — and **it cut the fan.**

M1 did not fan wide. What survived is evaluation for curve and surface,
derivatives, the interval path, insertion, splitting and conics, and four of
those are the same body of de Boor code reached through different doors. The work
ran as a gate and one long sequenced line, with two small pieces at the end.

This is a general finding, not an M1 detail. Rule 3 removes the independent,
textbook-oracled pieces — degree elevation, the Oslo algorithm, curvature and the
fundamental forms, arc-length parametrization — precisely because their consumers
are in later milestones. Those are exactly the pieces that were going to fan.
**Rule 2 currently reads as though this will not happen. It will happen at every
milestone.** Expect gate-plus-line, and treat a genuine wide fan as the exception
that needs arguing for.

### 16.7 The departures this milestone made

1. **No enum arm, and the `Copy` cost with it.** `BSplineCurve` and
   `BSplineSurface` are concrete types. There is no `Curve` enum and no
   `Surface::BSpline`. The arm is the seam to topology, its consumer is M2, and
   it costs `Surface: Copy` (`surface.rs`), which `FaceView::surface` consumes —
   rippling into the crate M2.5 holds for the duration. Deferred to M2.
2. **Clamped and non-periodic only, until M5.** Invariants 3 and 4 enforce it.
   Every periodic surface mechanical CAD cares about is analytic and stays
   analytic (roadmap §4.1); periodic NURBS first arrive with a STEP file, and the
   refusal message says so.
3. **One always-rational type, which vacates one of §14.5's oracles.** Dividing
   by exactly `1.0` is exact, so the non-rational case costs nothing and there is
   no second variant. That makes "all weights equal reproduces the non-rational
   B-spline" a statement about nothing. A deliberate loss, recorded rather than
   quietly dropped.
4. **§4.1 beats roadmap §4.5 on panics.** `evaluate` has exactly one failure
   mode — a parameter outside the domain or not finite — and returns `Err` for
   it. A zero denominator during evaluation means a validated curve carried a bad
   weight, which is a broken invariant, and it panics. Roadmap §4.5 says never
   panic; §4.1's three tiers win, because a corrupt point is worse than a crash.
5. **`publish = false` past M1.** §3.6 says "until M1", which now reads as
   optimistic rather than as a forgotten TODO. Nothing here is shaped for a
   stranger yet: `MAX_DEGREE`, the two roundoff constants and the conic
   constructors are all still finding their consumers.
6. **Rule 3 shrinks the fan rule 2 promises** — §16.6.

### 16.8 Known gaps carried into M2 and M3

- **Surface refinement does not exist.** Insertion, splitting and Bézier
  extraction are curves only. Nothing in M1 consumes the surface versions, and
  rule 3 says wait for the consumer. Roadmap §6.3's surface–surface marching is
  it.
- **General-angle arcs are not exact and were not built.** `cos θ` is an `f64`
  for almost no angle, so a general arc constructor cannot be exact, and an
  inexact one had no consumer this milestone. `circle` and `quarter_arc` are what
  shipped.
- **Conic exactness stops in two places, and the tests claim it only where it
  holds.** The in-plane frame is a cross product and a normalize, exact only when
  the axis is a coordinate direction; and a control point is `centre ± radius`,
  which rounds away from the origin. Both are documented at the constructor.
- **An enclosure costs 12× the `f64` evaluation for a degree-1 curve and 83× for
  a degree-5 surface** (332 ns against 27.7 µs). No consumer has asked for speed,
  so nothing was optimised. The first saving available is that the local interval
  divide computes four corner quotients where both operands are usually single
  numbers; it was left alone deliberately, because it is a correctness-critical
  divide and M3 is the first caller that will care.
- **The published `f64` bound is much wider than the enclosure it is tested
  against.** Across every family the enclosure's half-width never passed 0.145 of
  the bound, so the bound is at worst about seven times looser and usually far
  more. That is the honest behaviour of a bound stated over the homogeneous net
  when the net can be five hundred times the affine one, and it is another reason
  the bound is not a second enclosure source. It also means the ratio makes a
  usable scale-free ceiling where a count of `f64` steps does not — §16.1.

### 16.9 Review found what the suite did not, again

*Added after the milestone's adversarial review, which ran before the merge.*

§12.1 recorded that M0 shipped three bugs through a green 335-test suite and
that every one was found by review. **M1 did the same thing.** The suite was 549
green and `just` passed when the review started. It found seven defects, two of
them serious, and this section exists so the pattern is recorded twice rather
than rediscovered a third time.

**A certified wrong answer.** A de Boor coefficient's denominator overflows when
the knot range passes `f64::MAX`, every share collapses to zero, and evaluation
returns control point 0. On a degree-3 curve spanning `-9e307` to `9e307` that is
`0.0` where the truth is `1.5` — and `evaluation_roundoff_at` certified it with a
bound of `4.5e-14`.

This is **§12.2 in its third location**, and the shape is now unmistakable.
`EVALUATION_ROUNDOFF` is relative throughout, so it holds only while its
hypothesis does, and the hypothesis was enforced on the control coordinates and
on the weight but not on the knots. §12.2's own words were that we already knew
and guarded one of the three places the hazard appears; M1's plan called itself
the second; this was the third, in the same milestone, in a bound written by
someone who had read §12.2. **Enumerate every input a relative bound depends on,
and guard each one, or the count will keep going up.** The fix refuses such a
knot vector at construction, and the bound's doc comment now numbers all three
places rather than two.

The interval path was sound throughout and returned a box that does contain the
true point. That is the seam working exactly as designed, and it is the strongest
evidence in the milestone that making the enclosure the only enclosure — never
intersecting it with the `f64` bound — was the right call. Had they been
intersected, this bug would have produced a confidently wrong box.

**A silent NaN.** `tangent` returned `NaN` where the true derivative was exactly
`1e8`, because the derived net scaled before dividing. §4.1 ranks a silent
corrupt value below a crash, and this was one.

**Amended, after the fix was questioned.** Dividing before scaling left a
narrower case still refused — control coordinates more than the whole `f64`
range apart, whose true tangent may be an ordinary number — and it was written
down as a limit that *no ordering of the arithmetic could fix*. That was true,
and it was the wrong frame. Reordering cannot help, because every route to a
derived point needs the same subtraction; **scaling can**, and one halving
always suffices, since two coordinates each at most `f64::MAX` have halves at
most `f64::MAX` apart. The factor of two returns in the final scale. A tangent
of exactly `5e307` that had been refused now comes back exact.

The general lesson is worth more than the case: **a proof that no member of a
family of fixes works is not a proof that the thing cannot be fixed.** The
correct reading of "no ordering helps" was that ordering was the wrong family to
be searching. The gap survived a review that had already found two real bugs in
the same function, because it arrived carrying its own impossibility argument —
and an argument for why something cannot be done is exactly the kind of claim a
reviewer is least likely to re-derive.

**And once more, one layer down.** The same amendment left a second case
written down as a limit: `rational_derivative` forms `(A' − w'·C) / w`, and
`w'·C` can leave the range while the quotient would fit. Reaching it needs a net
built near the ends of `f64` on purpose, so it was recorded rather than fixed.
That was the same misjudgement a second time — "no consumer will hit this" is a
statement about today's callers, while a representable answer being refused is a
statement about the code. It is fixed the same way: when the product overflows,
`w` is by construction large enough to have brought it back, so the expression is
formed again dividing first, where `A'/w` and `w'/w` are both *smaller* than the
terms that overflowed. The two spellings are equal in exact arithmetic and fail
on opposite inputs, so the code tries rather than chooses — choosing would mean a
tuned threshold on `w` standing where a plain question about the answer does.

A curve whose true tangent is `-4.76e300` was being refused and now comes back,
matching exact rational arithmetic to every digit `f64` carries.

**What separates the two cases is worth stating, because it is the rule.** A
refusal is a *limit* when the answer does not exist in `f64` and a *gap* when it
does and the arithmetic could not reach it. A knot span one `f64` step wide under
ordinary coordinates has a true derivative past `f64::MAX` — that is a limit, and
`KnotVector::new` accepts such a span on purpose, so §4.1 makes it an `Err` about
the numbers asked for. Everything else was a gap. Neither fix claims totality and
the doc comments say so; what they claim is that the set of inputs answered is
strictly larger, and that the remainder refuses rather than handing back a `NaN`.

Both gaps were found by being asked about, not by the review and not by the
suite — which is the third time in this milestone that the thing carrying an
explanation of why it was fine turned out not to be.

**Three oracles that could not fail.** The knot-insertion oracle — §14.3's *first*
exactly-true relation — asserted nothing on about half of every family's draws,
because the helper choosing untouched control points returns an empty list on
short curves and only one of four tests checked. The split oracle's "two routes
to one answer" was one route: the by-hand raising loop was character for
character `split_at`'s own. Both are §14.1 question 2 answered "yes" by tests
that could not have said "no".

**And the cause of all of it: one generator family.** Every strategy drew its
knots from the same six small whole numbers, and the scaling oracle cloned the
knot vector unchanged while scaling only the control net — so the de Boor
coefficients, which are the dangerous code, received bit-identical inputs in both
runs of the only metamorphic test aimed at them. **No test in the milestone ever
varied a knot value.** That is why a bug in the coefficient denominator survived
549 tests.

§12.3 already said one family of shapes finds one family of bugs. What M1 adds is
that a generator can be varied in the dimension you are thinking about and
constant in the one that matters: the nets were random, wide-ranging and
carefully conditioned, and the knots were `0, 1, 2, 3` every single time. **Ask
of each generator not "is this varied?" but "which argument of the dangerous
function does this never change?"**

The repair is measurable. Knots now span 48 orders of magnitude, and scaling the
*knot vector* by `2^k` is a second oracle down a genuinely different path — the
point may not move one bit, the tangent must come back divided by exactly `2^k`.
Mutation confirms it: a de Boor nudge that only bites above `1e6` passes the
entire suite as it stood at review time, and fails all four of the new scaling
tests.

**What this says about §13 rule 6.** Rule 6 caps work in flight because review is
the constraint, not authoring. Both milestones now have the same evidence, and it
is stronger than the rule states: review did not merely find *more* than the
suite, it found things the suite was structurally incapable of finding. Budget it
as a phase with its own findings, not as a pass over finished work.

### 16.10 A claim that justifies an absence is not reviewable, and needs its own check

*Added after §16.9's second amendment, because the same thing happened twice and
the second time it was noticed only by being asked about.*

Two of M1's defects were found by neither the suite nor the adversarial review.
They were found by a reader asking what a sentence meant. Both sentences were
doing the same job, and it is a job no reviewer was looking at.

**The distinction that matters.** A code review examines **what is there**. Both
of these were arguments for **what is not there**:

- *"no ordering of the arithmetic gets that back"* — an argument that a fix does
  not exist.
- *"reachable only by handing `BSplineCurve::new` a net near the ends of the
  `f64` range on purpose"* — an argument that a fix is not needed.

An absence has no lines in a diff. A reviewer hunting defects reads the code that
exists, finds it consistent with the doc comment beside it, and moves on — which
is the correct thing to do with code. The review that ran on this milestone did
check both `assert!` unreachability arguments in `de_boor.rs` and found them
sound, because those are claims about **code that is there**. It walked past the
two claims about code that is not, and so would any reviewer, because there was
nothing to look at.

Worse, the reasoning attached reads as *evidence of care*. A bare `TODO` invites
scrutiny. A paragraph explaining why something cannot be done deflects it, and
deflects it in proportion to how well written it is. Both of these were well
written. Both were wrong.

**This generalizes §15.6, which the project already believes.** §15.6 says a
published bound needs **a test that could fail, not a paragraph** — prose
standing in for evidence, caught for one kind of claim. It is the same failure
here, in two more:

> **A claim that something cannot be done, or cannot be reached, needs a
> demonstration and not a paragraph.**

For an impossibility claim the demonstration is an enumeration: name the family
of fixes considered and say why that family is the only one. "No ordering helps"
was true of orderings and silent about scalings, and the silence is where the
bug lived. For an unreachability claim the demonstration is a reachability
argument through the **public API**, not through today's callers.

**Three species, and the question each needs answered:**

| Species | Sounds like | The question |
|---|---|---|
| Impossibility | "no ordering helps", "there is no way to reach it" | Is the named family of fixes the only family? |
| Unreachability | "nothing in the kernel builds that", "only on purpose" | Is it reachable through the public API at all? |
| Sufficiency | "one guard is enough", "by construction", "covers every case" | Enumerate what it ranges over; which member is uncovered? |

The third is not hypothetical either — it is the milestone's most serious bug, a
relative bound whose precondition was enforced in two of the three places it was
needed.

**The uncomfortable part: §13 rule 3 is also the camouflage.** "A part with no
named consumer is not built" is a real rule, it is in `CLAUDE.md`, and it is
right. It is also exactly the sentence the second defect hid behind. The
refinement, which §16.9 already names: **rule 3 governs features, not correctness
gaps.** Deferring a part nobody needs is rule 3 working. Deferring a case where a
representable answer is refused is rule 3 being used as cover, because the
question rule 3 asks — *who consumes this?* — is the wrong question to ask about
a defect. The test is §16.9's: a refusal is a **limit** when the answer does not
exist, and a **gap** when it does and the arithmetic could not reach it. Rule 3
may defer a missing feature. It may not defer a gap.

**Operationally**, and the half of this that lives in `CLAUDE.md`: the markers are
few and grepping for them is cheap — `cannot`, `never`, `always`, `no ordering`,
`nothing`, `only`, `impossible`, `unreachable`, `by construction`, `suffices`,
`covers`, `reachable only`. A whole crate holds a dozen or two. Each is a work
item with a definite question, and the answer is a demonstration or a rewritten
sentence. The `practices:claim-audit` skill is the pass, and it is deliberately
**not** a code review: pointing a reviewer at defects finds defects, and these
are not defects until someone asks what the sentence rules out.

#### A claim can also go stale, and the pass above will not catch it

*Added 2026-09-17, after one unit wrote the same defect five times in a day.*

Everything above is about a claim that was **wrong when it was written**. The
§110–§113 unit produced five of the other kind: **true when written, and still
written after the thing it described had moved.**

- `assemble.rs`'s `.expect("…already ruled out the one case…")` — true until a
  second error arrived beside the one it had ruled out.
- `one_step`'s `# Errors` naming the tangency variant — stale within hours,
  because §110.1 split the shortening budget off that path the same morning.
- §110.2's "**first** trial", asserted from an error that carries no trial
  index — so nothing could have contradicted it either way.
- A test whose name said a crossing was *only a rounding*, about a crossing that
  turned out to be real.
- A test named after one KiCad file, which §112 rescued out from under it the
  same day. The assertion was right and its fixture had moved (§113.7).

A sixth was sitting in this very section: the pointer just above used to read
`.claude/skills/claim-audit`, written when the skills lived in this repo and
left behind when they moved into the `practices` plugin. Corrected in place,
because a wrong path is not an argument to withdraw.

**What separates these from the three species above is time, not kind.** Every
one was a demonstration when it was written. What failed is that nothing tied
the sentence to the thing it described, so moving the thing left the sentence
standing — and a reader meets a well-written sentence and believes it, exactly
as the section above says.

The markers are different too, which is why this is a different pass and a much
cheaper one. The audit above greps for `cannot`, `never`, `by construction`. A
stale claim has no such word in it; what it has is a **name** — a variant, a
file, a count, a call site. So the pass is: *when you move a name, grep for the
sentences that carry it, in the same commit that moves it.*

**The rule is a test rule, not a prose rule.** A claim that names something that
can move has to be written so that moving the thing **fails something**, rather
than merely making a sentence untrue:

- Prefer a property over "whichever things do X", with a floor that goes red on
  an empty set, to a named fixture. A named fixture is a claim with a fuse in
  it, and the floor is what stops the property passing over nothing once the
  last case is rescued. §113.7 is the worked example.
- Select on the thing the claim is about, never on the symptom you are
  asserting. §113.7 again: selecting the files *by whether they named an entity*
  would have made a regression empty the set instead of failing the test.
- Do not assert what the evidence cannot carry. "First trial", read off an error
  with no trial index, is a sentence nothing can check. Carry the index or drop
  the word.

This is a sub-section rather than a section of its own on purpose. It is the
same rule as §16.10 — a claim needs something that can fail — applied to a claim
that decays instead of one that was born wrong, and splitting the two is how the
next reader finds only one of them.

### 16.11 The dynamic-range check is a ratio, and cannot see absolute magnitude

*Found by the §16.10 claim audit. The question it opened is answered in §18;
what stays here is what the audit actually established.*

`DynamicRange::is_within_supported_range` is `self.ratio <= LARGEST_SUPPORTED_RATIO`
and nothing else. So the check answers *"is this model's smallest feature
resolvable against its largest?"* — the question §3.5 set out to ask, and a good
one.

It is not the question three doc comments were citing it for. `build.rs` and
`validate.rs` both said `Undecidable` on exact `f64` input happens only when a
product overflows or falls subnormal, *"which is what the dynamic-range check
exists to catch"*. It cannot catch that, because the models where it happens are
**uniformly** tiny or **uniformly** enormous, and those have a ratio of exactly
one. Measured: the `1e-170` square that used to panic the profile builder
reports `ratio = 1, accepted = true`. Two further copies of the same sentence
were later found in `orient2d.rs` and `orient3d.rs`, which the freeze had put
out of reach of the first correction — see §18.

**Correction, and it is this section's own mistake.** As first written, the
paragraph above said `largest_coordinate_magnitude` is "computed, stored, and
never consulted". That is false. It has no accessor, but `coordinate_spacing()`
reads it inside the same module to build the `Warn`-severity slop tag. It is
neither dead nor a §13 rule 3 violation, and the sentence asserting otherwise
was written in the same session as §16.10 and never checked. Recorded rather
than quietly edited, because a section about unchecked claims making an
unchecked claim is the most useful example it could have.

**The design question this opened — should the kernel refuse a model whose
coordinates are absolutely too small or too large, independently of their
ratio? — is answered "no" in §18**, and answered by removing the reason to ask
it rather than by weighing the trade. Uniformly small or large coordinates turn
out to be a §16.9 *gap*: the sign exists and an exact power-of-two rescale
reaches it. A predicate that can decide them has nothing to refuse.

**The inversion worth carrying forward.** The ratio check catches the genuinely
wide-spread model, which is a real *limit* — no single scaling brings the
differences and the raw coordinates into range together. It is blind to the
uniform case, which is not a limit at all. So the check was right about what it
caught, and every sentence citing it was reasoning about the other half.

**The large end is a different bug, and §18 does not fix it.** Measured:
`orient2d` and `orient3d` never fail on the large end at all, out to `1e200`.
The `1e140` cube's `classify_point` failure comes from the ray-cast target,
derived from the model's bounding box, overflowing — not from predicate degree.
It lives in `rcad-topo`, it has its own cause and its own oracle, and §13 rule 6
says it is a separate piece of work.

Recorded with a hypothesis attached, so the next pass starts somewhere: it is
probably a gap too, and a ray target chosen relative to the bounding box rather
than in absolute coordinates would likely just work.

**Both of those paragraphs are wrong, and §20.7 has the measurements.** The
`1e140` cube classifies `Inside` with an empty log and has since `a23d73d`, so
there is no large-end `classify_point` failure left to explain. The cause named
above was never the cause: the ray target does not overflow until side
`7.7e307`, and at `1e140` what overflowed was inside `orient3d`. The hypothesis
attached for the next pass therefore proposed a fix for a problem that did not
exist — a reminder that a cause recorded without a demonstration is a guess
wearing a measurement's clothes, which is the whole of §16.10.

One thing that made this urgent has also gone quiet in a way worth noting.
`Model::validate` does refuse the `1e140` cube — but by an unrelated
`VolumeIsNotPositive { volume: inf }`, not by its dynamic-range step, which
passes with a ratio of one. The protection was always accidental. **§20.4 acted
on that sentence:** an infinite volume is no longer reported as an invalid
model, because a `1e300` cube is not broken, and `Invalid::VolumeIsNotPositive`
now means only that the faces point inwards.

