## 48. Nothing round was ever checked — the unit after the six arms, recorded 2026-09-04

*Written before any lane was dispatched. Every file and line it cites was checked
against the tree at `0febd16`, by opening the file and not by trusting the report
that named it — two of the citations below changed when that was done, and one
correction was withdrawn outright. The corrections come first, for §46's reason:
a lane that reads a stale price scopes itself wrong on its first day.*

### 48.1 The name, and what the unit is

Called **nothing round was ever checked**, after the gap and not after the fix.
§46.7's rename is the precedent, and the reason to name a unit for its gap rather
than its hope is that a name stating a gap cannot be quietly satisfied by a lane
that widens one fixture list.

**`the cut` stays held.** §46.7 shelved it once for the unit where the conic
drawing landed; the drawing landed and the cut did not. It is shelved a second
time here. A name that has never been earned should not be spent early.

The unit's spine moved during planning, and it is worth saying why. It began as
the ordering work §47.7 names — *"nothing can put points in order along this
curve"*, ninety times — and that is still the kernel's missing capability. What
displaced it was reading what the fuse's own oracle actually covers.

### 48.2 Five corrections, before any lane reads them

**1. §47.7's *"cones are absent from `fuses_that_must_agree.rs`"* is understated,
and the accurate sentence is much worse.** Every property in that file that
compares invariants — inclusion–exclusion, idempotence, absorption,
associativity, the multi-argument rows and the difference identities — calls
`invariants_of` or `volume_of` at lines 367 through 840. Every use of the one
curved fixture is at lines 1128, 1183, 1222, 1275 and 1316. **The two sets do not
overlap at all.** The curved family runs **commutativity and nothing else**,
plus a pin on today's refusal and a disjoint case that answers nothing.

Transform invariance is worse than absent: `Answer::moved_by` **panics** on a
curved face (`fuses_that_must_agree.rs:2156-2159`), unconditionally on kind, so
even a pure translation of a ball would panic. Its own doc says the whole of the
obstacle — *"nothing here has worked out what it is on a pipe's axis."*

So roadmap §11.2's instruction — *"**Start with transform invariance.** Kernels
overfit badly to axis-aligned and origin-centred configurations"* — has never
once been carried out on a round face. Not on a cone. On a **cylinder**, which
has been in this kernel since M2.

This row has now been written down three times, each smaller than the truth:
*"cones are absent"*, then *"cones and spheres are absent"*, then the sentence
above. Each correction came from looking one level deeper at a row somebody had
already noticed and moved on from. That is §48.9's prediction.

**2. §46.4's named finding fired, and it is in the tree but not in this file.**
§46.4 said that if closing the hyperbola did *not* break `refused > 0` on the
corner-up-cube families, *"that is the finding — the hyperbola was not what
refused them."* It did not break, and
`how_often_a_fuse_with_a_cone_refuses.rs:600-618` records the whole thing: a
frustum answers 9 of 96 with 27 `AWireWillNotLieFlatOnItsFace` and 60 undecided,
a round bar of the same size in the same place answers **9 of 96 too** with 21
and 66, and therefore *"what those 87 refusals measure is a hard family, not a
hard surface."*

**The correction is to the correction.** This section first said the finding was
never recorded. It was — in the test file that measured it. What is true is
narrower: §47 does not carry it, so `decisions.md` alone does not tell a reader
that a cone is no worse than a cylinder on that family. Recorded this way because
the first draft of a corrections section getting a correction wrong is exactly
what §46.8 item 1 was, and the cheapest demonstration that reading the file
beats reading the report about the file.

**3. §47.3 promised to score option D's trigger and did not.** §44.6 states the
trigger so it can fire: *D fires when, after the named gaps close, more than half
the still-refused files are refused for a miss larger than `f64` rounding*,
scored there at 50% to 56% — on its own boundary. §47.3 says the trigger *"is
scored on the published number"* and then publishes the number without the
verdict. A fallback whose condition is never evaluated is a fallback with no
condition, which is the thing §44.6 was written to stop. **The closing gate
scores it.**

**4. `ACrossingThisCannotBeOrderedAlong` is two unrelated problems under one
name.** Its doc (`rcad-topo/src/error.rs:563`) cites ball × pipe and pipe × pipe
— a quartic nothing can order. The ninety frustum refusals are an open **conic**,
which is a different problem with a different fix. Any unit that moves either
count reports a number that mixes them.

**5. The laws suite cannot see an asymmetric non-manifold answer.**
`Err(Error::ResultIsNotManifold { .. })` returns `Ok(None)`
(`fuses_that_must_agree.rs:1572-1575`) on **every** family, including the ones
where an `Undecidable` refusal is a hard failure. So `A ∪ B` coming back
non-manifold while `B ∪ A` answers is silently skipped. That is
order-dependence, invisible to the suite whose job is finding it.

*Later:* the suite counts it now, and §351.6 closed the turned family's last
known cause. The frame and comb that first showed it share the same two
solids both ways round, and `both_agree_about_whether_there_is_an_answer`
fails on the turned family instead of counting.

**What is not wrong, and was checked.** The vacuity guard is sound:
`SOME_COMPARISONS_MUST_HAPPEN = 0.3` (`:274`) with a `meant_to > 0` guard, so it
is not §47.4 item 1's shape. The file's own doc says the risk out loud at
`:9-13` — *"a green run says nothing on its own."* And the suite already picks
correctly between the bit-exact fingerprint and the invariant comparison per
property, which is better than roadmap §11.2 warns about.

### 48.3 The sentence the unit earns

> **Every algebraic law in `fuses_that_must_agree.rs` runs over a round solid,
> and transform invariance runs over one first.**

Falsifiable by counting: open the file, list the properties, and check each
one's fixtures for a curved entry. The second clause is roadmap §11.2's
instruction, followed four milestones late.

### 48.4 Wave one, and what each lane may write

| lane | one deliverable | may write |
|---|---|---|
| **L1 the fingerprint** | curved `moved_by`; the non-manifold asymmetry arm; wire curved fixtures into the properties that have never run over one | `tests/fuses_that_must_agree.rs` |
| **L2 the fixtures** | a ball and a frustum, with strategies, to signatures fixed before dispatch | `tests/pairs/mod.rs` |
| **L3 the claim audit** | claims of §16.10's shape in the two fuse files; the tangent-ruling pair is the seed | `src/fuse/arrange.rs`, `src/fuse/assemble.rs` |
| **L4 the seam question** | does a disjoint curved union validate, measure and classify right, and hand back a face with no seam? The **question**, not the fix | one new test file |
| **S the spike** | whether an open bend has a monotone key | scratchpad only |

**The ball comes before the cone.** A cone mostly refuses, so it buys vacuous
passes; `add_ball` is a public one-liner (`build.rs:719`) and a sphere has been
fuseable since M4. *"The laws have never run over a sphere"* is the more
embarrassing sentence and the cheaper one to fix.

**L1 and L2 are coupled by an interface, so the dispatcher fixes the fixture
signatures before either starts.** A lane inventing an interface the other lane
must guess is how two lanes become one.

**L4 takes the question and not the fix** because roadmap §11.2 says topological
equality is the wrong predicate — two correct kernels partition faces
differently. §47.7's dropped seam is a defect only if the answer validates while
carrying a face nothing cuts open, which is what
`ACurvedFaceGoesRightRoundWithNoSeam` exists to refuse. **That answer decides
whether the seam is this unit's top priority or a decomposition note**, and
guessing which would be picking the branch that suits the plan.

**Two rules whose pass and fail are written before the lane runs**, because §47.6
records that the brief which produced the best work was the one that said what
counted as failing:

- **L1's fingerprint stays bit for bit, or says why not.** A quarter turn about
  `z` is a swap and a negation (`pairs/mod.rs:259-279`), exact on a ball's
  centre, a pipe's axis and a cone's apex, and `Surface::transformed`
  (`rcad-geom/src/surface.rs:275`) already carries every scalar over untouched.
  **Pass:** the curved families pass `moving_a_pair_leaves_the_answer_alone` with
  no tolerance introduced anywhere. **Fail:** any bit difference — and on fail
  the lane names *which coordinate differed and why*, and drops that family to
  the invariant comparison while calling that a weakening in the doc rather than
  doing it quietly. The suite's one distinguishing property is that it has no
  tolerance in it; that is not traded for an afternoon.
- **The spike's rule.** The proposed key is `sign((p₁ − p₂) · (n × out))`, which
  drops a hyperbola's `cosh` term only if `out · (n × out)` is exactly zero. It
  is zero as a polynomial and merely near zero in `f64`, so the contamination is
  bounded by the walk's own `cosh σ ≤ 2` cap and the interval machinery reports
  `Undecidable` rather than lying. **Pass:** every sampled pair on one branch
  decides under a certified policy, and the order agrees with the parameter
  order, with zero disagreements. **Fail:** one undecided pair on a
  well-separated configuration, one disagreement, or thirty minutes gone.
  *"The contamination looks small"* is a fail.

### 48.5 The budget, with the gates reserved first

**9 agent-hours, of which two gates take 2.0, reserved before the first lane
starts.** The spike takes 0.35; what is left buys lanes at about 0.75. Wave one
is about 5.35 and wave two is composed from what wave one reports, not planned
here.

Two gates and not one, deliberately rather than by discovery: §47.4's two gates
between them found five false published sentences that twenty lanes missed, and
§47.6 records that **a lane told to justify a sentence found the sentence wrong
more often than a lane told to check code found a bug.** That is the best-value
time in the last unit and it is not the part that gets squeezed.

**Drop order: the spike, then the claim audit, then the seam question. L1 and L2
never drop, and never drop separately.** The spike goes first despite being the
cheapest thing here, because it is the only item whose result changes nothing in
*this* unit — it is pure input to the next one, and when hours are short the
thing to cut is the value that is entirely deferred.

**L1 as scoped is three deliverables and CLAUDE.md caps a lane at one.** It will
probably split. Its brief carries the commit-at-halfway rule and a *what is left*
section either way, per §47.6.

### 48.6 Two decisions that were made twice and disagreed

Written here rather than discovered later, which is §46.13's lesson applied
before the fact instead of after it.

- **The error-name split is not in this unit.** It was first settled as *"split
  it regardless of the spike"* and later as *"it pairs with the ordering lane."*
  The later form wins and the lane list has no slot for it: if no count moves
  this unit, splitting the name that reports the count is not yet urgent. It
  goes forward with the ordering lane.
- **The third way a crossing can run is decided and not built.** `HowItRuns`
  (`fuse/arrange.rs:448-467`) has two variants — a coordinate sort for endless
  straight runs and a cyclic order for closed curves — and an endless *curved*
  run has no representation at all. The decision, recorded now and built next
  unit, is to **add a third variant and leave `AlongAnAxis` alone**: its axis is
  chosen by `dominant_axis_of_a_direction`, which refuses when no coordinate has
  a provably nonzero component, and that guard is what catches a plane resting
  along a pipe's own side. Generalising it would put working, certified,
  guarded arithmetic behind new arithmetic to serve a case that does not need
  it.

### 48.7 What stays out

| out | why |
|---|---|
| **the ordering lane** | the spike runs; the lane does not. It wants `arrange.rs`, which the claim audit is reading, and it is not this unit's spine |
| **cone against pipe** | roadmap §13 row 1, and it is what blocks `AFrustumAgainstABar` and `TwoFrustumsOnOneAxis`, both pinned at nothing answered. **This is the next unit's headline** |
| **torus** | all five of its families answer nothing and all five refuse on quartics. Untouched by any of the above |
| **the writer** | §46.8 item 3, unchanged. A ball that imports cannot be written out |
| **fillets** | §46.5's reason stands |
| **another import-rate push** | option D is *scored* here, not acted on |

### 48.8 Two rules this unit adopts, and one of them is permanent

**Into §15, as a standing rule every later unit inherits:** *for every rate that
moves, name the cases that moved and say what they have in common.* §47.8 found
it by accident and §45.7, §46.9 and §47.2 are three consecutive units whose
central lesson was the same gap between a measurement and the claim it was
offered as evidence for. Three in a row is structural, and the per-unit form has
now been tried three times. **The closing gate may not accept a moved number
without its case list.**

**And this unit's own:** an open-table row must say what it **measured**, not
what it **noticed**. *"Cones are absent"* is noticed. *"Every property comparing
invariants runs at lines 367 to 840 and every curved fixture at 1128 to 1316,
and the sets do not overlap"* is measured. §48.2 item 1 is what the difference
costs.

**The corrections section is audited like anything else.** §46.8 item 1 was a
mis-priced correction sitting inside the section named for corrections, and
§48.2 item 2 is one this unit caught on itself before publishing. The closing
claim gate reads §48.2 with the same suspicion it reads a doc comment.

### 48.9 The prediction

§47.8 predicted a correct number with a false sentence attached, and Q3's rule
above is the countermeasure. **The prediction for the unit after this one is
about what this planning session actually demonstrated:** it loses to a row in an
open table that is *true but too small*. §47.7's cone row was written down three
times, each time accurately and each time smaller than the fact, and every
correction came from asking what the row had been measured against rather than
reading it again.

So: **the next unit's largest finding is already written in §47.7 or §48.7, in a
row nobody has priced.** The countermeasure is §48.8's second rule, and the way
to score this prediction is to check, at the next gate, whether the unit's
biggest surprise had a row waiting for it.

### 48.10 The spike passed, and the risk it was written for does not exist

*Added after the spike ran. §48.4 is left exactly as it was written — rewriting a
pass rule to match a pass is the discipline this unit is named for.*

**PASS**, against §48.4's rule and with no part of it relaxed.

| curve | ordered pairs asked | undecided | out of order |
|---|---|---:|---:|
| parabola | 4 920 | 0 | 0 |
| one branch of a hyperbola | 11 480 | 0 | 0 |

All under `Policy::certified()`, all settled by intervals, with the exact level
never reached. Fixtures span cones of slope 0.25, 1 and 4, and planes through
the origin and 500 units out.

The key is what §48.4 proposed:

> `sign((ahead − behind) · (n × the_bend's_own_way))`

where `the_bend's_own_way` is a parabola's `bend` or a hyperbola branch's `out`,
and `n` is the cutting plane's stored normal.

**The contamination §48.4 budgeted for is not small — it is absent, and the
reason is a spelling.** §48.4 wrote that `own_way · (n × own_way)` is *"zero as a
polynomial and merely near zero in `f64`"*, and reasoned that the walk's
`cosh σ ≤ 2` cap bounds what that costs. That reasoning applies only if the cross
product is built as three `f64`s first. **Kept inside `Exact` as a determinant,
the triple product returns `Decision::Zero`, asserted, on every fixture.** It is
the choice `HowThreeOnACurveWind` already makes about not building its fourth
point, one level further down.

The built-vector spelling would also have passed — the worst own-way term
measured is `4.65e-17` against a signal of `9.95` — but that is luck and not a
cancellation, and the determinant is what the ordering lane should write.

**The check has teeth, which is the number that makes the zeroes worth reading.**
Aimed down `across` instead — a direction the quadratic term does *not* cancel
against — the identical tally reports **404 out of order and 13 undecided out of
820**. Resolution, found by halving the gap until it refuses, is about `1e-16`
relative on both curves: the `f64` grid itself, not a margin the triple product
eats into.

**What it does not show, stated because §48.8's rule cuts both ways.** Every
sample is an exact point; `arrange.rs` passes boxes at three-surface corners, and
nothing here says a wide box decides — though every pair settled at level 2 with
the exact level unused beneath it. And only plane-against-cone was tried, because
it is the only pair that makes an open bend today; `n` being a *plane's* normal is
part of the key, so an open bend from a pair with no plane in it needs its own
derivation.

**So the ordering work is priced, and it is not roadmap §13 row 1.** What stands
between a block and a frustum is one `HowItRuns` variant (§48.6), one predicate
that now exists in production shape, and the call sites. The predicate is kept on
the branch `worktree-agent-a1820cae9666ed345` rather than thrown away with the
worktree.

