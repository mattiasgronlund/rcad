## 51. What the endless bend cost and found, recorded 2026-09-05

*Written after the wave, from two closing gates' verdicts rather than from the
lane reports. Where a gate and a lane disagreed, both numbers are given and the
one that was re-derived from the tree wins. Seventeen lanes across four waves,
fifty-two commits, `just default` green twice at the end.*

### 51.1 The sentence, and which half was earned

> **The fuse can put points in order along a curve that never ends, and a cone
> coaxial with a pipe is cut by it.**

**Both of §50.3's falsifiers are satisfied.** `HowItRuns` has a third variant,
`AlongABend`, with a construction site that is not a test; `intersect` no longer
refuses the coaxial pair, and `walk` draws the two circles.

**The consumer the unit was built to prove still answers nothing.**
`AFrustumAgainstABar` is 0 of 96, because the fuse now gets past `intersect` and
stops one step later — *where three surfaces meet is only worked out when two of
them are flat*, and a cone, a pipe and a cap plane give one flat surface. §50.6
puts `three_surfaces.rs` out of scope, so this is the next wall and not a gap in
the work. The pin now carries the new reason and fails the day that wall moves,
which is the only thing that makes a pin holding at zero worth keeping.

### 51.2 Four defects nobody planned for, and what each was hiding behind

None of these is the headline. All four came from a lane or a gate contradicting
the brief it was given.

1. **A fuse refused solids that provably have volume.** `nest_the_shells` decided
   *solid or cavity* on the sign of a rounded sum of triple products —
   `signed_volume` accumulating `a.dot(b.cross(c))` in plain `f64`, with no
   `Decision`, no `Interval` and no exact fallback anywhere on the path. A prism
   over a triangle three ulps off its own long side, whose area `orient2d` proves
   `Pos`, measured `-2.3129646346357427e-18`: **2020 of 20 000 draws refused for
   having no volume.** A square rod `1e-9` across down the `(1,1,1)` diagonal,
   corners as well conditioned as a cube's, measured `-4.5e-18` of `1e-18`.
   Fixed by keeping the rounded sum when a forward bound proves its own sign and
   adding the fan up in `expansion` arithmetic when it does not.

   **Why the existing defence never covered it, proved rather than observed.**
   Every term of a 3×3 determinant carries exactly one factor from each axis, so
   shrinking an axis by a power of two shrinks every term by that power and the
   sum returns bit for bit what it was, scaled. That is what makes `axis_shifts`
   free, and it is the same fact that makes it useless against cancellation: it
   rescues a solid thin along an **axis** and does nothing for one thin along a
   **diagonal**. The guard is also downstream of the decision it would protect —
   `check_dynamic_range` is twenty-three lines and one conditional after
   `nest_the_shells` has already believed the sign.

2. **One surface, two names, from a signed zero — three surfaces in a row.** The
   kernel states *two faces that are the same surface hold the same numbers* in
   four places — `Surface::how_it_sorts_against`, `name_of_surface_bits`,
   `canonical_cut_surfaces`, §32.5 — and all four compare with `total_cmp`, where
   `-0.0 < 0.0`. It enforced it in none. A shared wall put a corner in two places
   one ulp apart. Fixed at construction in `Plane`, then `Axis`, then `Sphere`,
   each by the lane fixing the one before it, each stopping at its write-set
   boundary and handing back rather than reaching. `HalfSpace` is exempt by
   §33.8 — it has no canonical form at all.

   Two measurements worth keeping. Normalization folds **none** of it and can
   **manufacture** a negative zero: a component small enough to underflow against
   the largest (`-5e-324` beside `1e300`) comes out `-0.0` where nothing in the
   input was a zero. And the way in is not exotic — **335 of the 367 corpus
   files hold a `DIRECTION` one of whose components is a negative zero, 16 770
   entities of 283 349**, spellings like
   `DIRECTION('',(-0.0,-0.0000000001,-1.0))`, which `direction_at` puts straight
   into `Vector::new` and `placement_at` hands to `Axis::new`. The criterion is
   *a component token beginning `-` that parses to `0.0`*, over `DIRECTION`
   entities matched across line breaks. **This paragraph first said 93, and
   §51.6's prediction fired on
   §51 before the ink was dry**: 93 came from a lane report, was quoted here
   without its criterion, and no criterion anybody tried reproduced it — four
   different greps gave 340, 112, 359 and 335. Which is why the rule below is
   about naming the command and not about trying harder.

3. **A soak that reported 4.29 billion cases and ran 2049.** `MEASURED_CASES *
   factor` as a bare multiply, in five places across two crates. In release it
   wraps, so the run is silently narrower than shipped while the operator reads
   the wide number — §48.2's subject, in the machinery built to answer §48.2.
   `checked_mul` throughout, each site demonstrated wrapping before it was fixed.

4. **A soak that could not soak.** Once the multiply was fixed,
   `the_same_pair_turned…` still died at `FACTOR=50` on proptest's
   `max_global_rejects`, a flat 1024 whatever the case count, **before any
   assertion about geometry ran**. Scaled to proptest's own four-per-case ratio;
   ten seeds now compare 2.05 million turned pairs.

### 51.3 The fuzz corpus had stopped growing with the kernel

`two_surfaces_from` reads two bits per surface and asks `which % 3`, so the
824-seed corpus has only ever held planes, balls and pipes — decoded and matched
on `Surface`: **232 planes, 423 balls, 831 pipes, 0 cones, 0 tori**. Bits 4 to 7
of the kinds byte are read by nothing, and `3 % 3` aliases to a plane, so the
draw is 2:1:1 and not uniform. Every claim in the crate that says *over the fuzz
corpus* was a claim about three of five surface kinds.

**The corpus is not repacked, and the second reason is the load-bearing one.**
Promoting the unread bits reinterprets 92 of 824 minimised seeds — not "every one
of them", which is what the file said. But a torus reads eight numbers where a
pipe reads seven, so a torus against a cone, a pipe or another torus needs **76
bytes and the longest seed checked in is 71**. Those pairs would decode to
nothing whatever the kind bits said. Widening the decoder is a lane of its own
and wants the corpus re-minimised with it.

Two oracles landed meanwhile — `intersect(a,b)` against `intersect(b,a)`, and the
same pair written at `2^±1` — closing a hole the tree had no way to see: **a pair
the fuzzer refuses to name produces no point to check**, so every defect that
shows up as a refusal was outside what any amount of fuzzing could reach. Cones
and tori are covered by a separate hand-written table of 136 pairs on the integer
lattice, reported as its own row and never blended (§17.7).

### 51.4 Six false sentences, every one beside a number that was right

This is the unit's finding about itself, and it is §47.8's shape for the fourth
consecutive unit.

| the sentence | what was actually true |
|---|---|
| §50.5: the frustum family answers **84 of 84** | §49.6's 84 was configurations on which a **rule** held, never answers. Pre-wave: 38 answered. Post-wave: **53**, with 0 refused for want of an order where 44 were, and all 31 remaining stopped in `select.rs` |
| §50.2 item 4: naming beyond the coaxial pair is unsafe **because of cone-against-cone** | a **non-coaxial cone and pipe can share a line** — a pipe of radius 2 about `(1,0,1)` through `(0,2,0)` holds the ruling `(t,0,t)` of `Cone::new(Axis::z(), 1.0)`, since `‖(p−o)×d‖² = 8 = 4‖d‖²`. Two families rest on `intersect`'s refusal, not one |
| a lane report: the parabola knife edge is **not reachable with rational numbers** through the public door | `E` is homogeneous of degree 2 in `n`, so normalizing divides it by `L²` and moves no sign, and the division rounds exactly at **power-of-two ratios**. `Cone::new(Axis::z(), 0.5)` against `Plane::from_point_and_normal((0,0,4), (−1,0,0.5))` is a parabola with whole-number points |
| `NO_POINT_MAY_USE_ALL_OF_ITS_BOUND`: the worst a search finds is about **3·ε** against a published 256·ε | true of three planar rows. The quartic control sits at **0.738 of its bound**, about 189·ε; the headline 0.429, the band 0.477 and rising with corpus width |
| the fuzz file: repacking would reinterpret **every one** of the 824 seeds | **92**, or 42 for the narrower repack |
| `plane.rs`, mine: turning a wall round twice is the plane it started as, **bit for bit** | false for a plane storing `-0.0`, which my own fix had just made possible |

Two of the six were written into §50 before any lane ran, and both were the same
error: **quoting a row from an earlier section without opening what it measured.**
§50's preamble says every file and line it cites was checked against the tree by
opening the file. That was true of the file citations and false of the numeric
ones, and the distinction is the whole finding.

### 51.5 §49.7's prediction fired, on one site

> *"The next unit will ship a correct fix that carries a rounded test into a place
> where it decides an answer."*

**Fired, on the presort and only there.** Six of the seven call sites the ordering
predicate reached carry their own derivations and hold up under reading. The
seventh — the hint sort in `nodes_along`, which the building lane's own table
omitted — is a rounded `total_cmp` justified by a sentence inherited verbatim from
the straight-crossing case: *"every neighbouring pair is then proved."* It was
not. `merge_points_that_are_one_point` asked and acted **only on `Decision::Zero`**,
discarding a `Neg`. A gate swapped one pair in the hint and got **8 tests failing
with wrong answers, not refusals**.

Nobody could make the rounded key actually wrong — instrumented, the topo suite
stays green at soak factor 8 with 1024 proptest cases, and the instrument fires
immediately when inverted. So there was no demonstrated defect and no proof
either, and the sentence claimed one.

**The fix is not the assertion the gate proposed**, and the reason belongs in the
record. The hint is a rounded read of the **caller's** coordinates, so asserting
on it puts caller data into a panic; and `order_along` is holding the right answer
at that exact line, which makes it §16.10's **gap** — refusing over an answer
already in hand is the family the rule names. `prove_the_hinted_order` turns the
hint into the order by insertion sort over the proved comparison. The gate's swap
experiment, re-run against the fix, changes nothing: 586 of 586 pass with the hint
deliberately scrambled, for straight crossings and for bends.

### 51.6 The prediction for the next unit

Six sentences, four consecutive units, and the countermeasures already tried are
§15.10's case list (works), the fail-condition brief (works), and the closing gate
(works — it caught five of the six). What none of them reaches is a sentence
written **before** the work, in the planning section, quoting an earlier unit.

> **The next unit's largest error will be inherited from a section it quotes, not
> produced by a lane.** Score it at the next gate by taking every number §51 or
> the next planning section cites from an earlier §, opening what that § actually
> measured, and checking the reading. The countermeasure is cheap and specific:
> **a planning section may not cite a number from an earlier section without
> naming the test or command that produced it.** A row that cannot name one is
> quoted as prose, not as a bound.

**A claim that other claims rest on is stated once, in a fixed phrase, and cited
rather than restated.** The test of whether a load-bearing sentence is written
well enough is whether one grep finds every place that leans on it.

That is the half the rule above was missing, and it has a demonstration rather
than an argument behind it. *Grep for the reason, not the conclusion* is useless
when the reason was paraphrased three different ways and only one of them sat
near the distinctive word. It happened in `rscene`'s design doc, to the sentence
"coarser is likelier to succeed": written three times in three wordings, killed
by this unit's pointed cone, and retired in exactly the one place its editor
happened to be working. The other two survived their own evidence, and were
found by luck.

### 51.7 What stands, and what this unit made more likely dirty

| open | where |
|---|---|
| **`three_surfaces.rs` wants two flat surfaces** | the new wall for `AFrustumAgainstABar`, 96 of 96, pinned with its reason. §50.6 kept it out |
| ~~**one pipe, two names, from an axis written either way**~~ | **closed by §51.9.** The geometry key folds a pipe's, a cone's and a bend's axis direction as it already folded a plane's facing |
| **one pipe, two names, from an origin slid along its own line** | the half §51.9 did **not** close, because folding it needs a projection and a projection rounds. A cone's apex and a bend's centre cannot slide, so it is the pipe alone. Masked today — two solids on one pipe refuse at *a point inside a piece of a face* before a corner is named, identically whichever way it is spelled — and pinned so that lifting that refusal breaks the pin rather than quietly reopening this. **134 corpus files hold two slid spellings of one pipe and 9 hold them on faces that share a corner**, against 36 and **0** for the reversal, so the half left open is the half real files actually reach |
| **`the_way_it_is_parametrised_about` is odd in the stored axis direction** | `validate.rs`, so a cap circle reads as running the other way under a reversed pipe. Not an identity question and the fold does not touch it, but it is the one place two spellings of one pipe are genuinely not interchangeable, and whoever moves the coincident-pipe wall meets it |
| **`fuse(A, B)` and `fuse(B, A)` refuse with different provenance over placed solids** | one records `Exact`, the other a `Constructed` box `5.7e-14` wide, on solids built with `Isometry::translation` where the builder-built control records `Exact` both ways. Both refuse, so no wrong answer — but §17.4's bit-for-bit agreement is about answers and says nothing about records |
| ~~**`room_left_over` waves a walk through and records nothing**~~ | **closed by §52.2.** A wave-through now pushes a record. The row's second clause was **false** and §52.4 says why: a caller could already tell the two apart, because every other exit from `walk` either floors its step count at two or carries a zero deviation |
| ~~**what the mesher does with an empty walk list on a *closed* edge**~~ | **answered by §52.3, and it is not a defect.** Two independent barriers refuse the mesh before anything is published. Scoped: *no route the suite can build*, not *never* — see §52.7 for the route nobody reached |
| ~~**the pointed cone refuses at 1e-3 of its own size with meshes either side**~~ | **closed by §52.5**, and it was a §16.10 gap in the cone's bound, not a property of the shape. The bound charged a **ruling** — a triangle with two corners on the apex, lying exactly on the cone — as though those corners sat on the base circle |
| ~~**an oblique plane cutting a cylinder**~~ | **struck by §52.6: the row is false, and backwards.** The refusal does not fire at any tilt on a solid `build.rs` builds. The tilt makes a boundary side *cheaper* by `cos θ`, and the worst consumer of the boundary budget is the flat un-tilted circle. The one-line fix it expected was already in the tree. What is real in that geometry is a different quantity — §52.7's leaning-cap row |
| **a mis-signed shell reaching `nest_the_shells`** | both families that flip the sign refuse earlier in the arrangement. Neither barrier is principled — the rod's union answered at `1e-6` and `1e-7` — so it is unproven in both directions. The fix removes the question rather than settling it |
| **`how_much_space_a_curved_solid_encloses`** | in `validate.rs`, still compares refinements against a rounded `f64`. The sign it reads is now exact; the comparison is not audited |
| **the fuzz decoder draws 3 of 5 kinds** | §51.3. Wants the corpus re-minimised with it |
| **`cuts.rs` and most of `three_surfaces.rs`** | still **unaudited**, and now **more likely dirty than before**. §49.8's class was found a fifth time this unit, in the file next door. A comparison-operator grep over both is nearly empty — and that is not evidence, because **it is the same grep that missed the volume sign**, where the rounded sum sat three call-levels away behind a function returning `f64`. Whatever audits them must follow every `f64`-returning call |
| **`ResultIsNotManifold` with `count` other than 1** | unchanged, and now with 9 named draws from the `AFrustumAgainstABlock` family |
| **the writer, fillets, the torus, cone against cone** | unchanged |

### 51.8 What was learned about running the work

- **Five lanes at once, then three, three and one.** Write sets were named before
  any lane started and every merge was clean — no conflict in seventeen lanes.
  The cost of that discipline is that a lane which finds a defect one file outside
  its set must hand it back; the benefit is that four of them did, and each
  handback became the next lane's whole deliverable.
- **Briefs that name what counts as *failing* keep working.** Third unit running.
  Every one of §51.4's corrections came from a lane or gate contradicting its own
  brief — which only happens if the brief says what a wrong answer would look like.
- **Two gates, split by numbers and by claims, beat one gate.** Each found things
  the other did not, and each disagreed with a lane. A gate is also not exempt:
  the claims gate published corpus kind counts that did not sum to `743 × 2`, and
  the lane that re-derived them said so and showed the arithmetic.
- **A `sonnet` lane staged its whole deliverable and never committed it.** The
  brief said commit as you go; had it been killed, all of it was lost. Worth a
  hook rather than another sentence in a brief, on §50.8's reasoning that
  recording a rule has been tried.

### 51.9 One pipe, one name — the reversal folded, the slide left open

Run after §51 was written, on the row §51.7 listed second.

**The false sentence is the finding.** `name_of_the_surface_either_way_round` —
the helper written for exactly this problem, which already folded a plane's
facing — ended its doc with *"A ball and a pipe have no reversed form at all and
are their own key."* A ball genuinely has none: its centre and its radius are all
it holds and neither has a sign to turn round. **A pipe has one, and so do a cone
and a bend.** `Axis::reversed` keeps the origin and negates the direction, so
`Cylinder::new(Axis::z(), 2.0)` and `Cylinder::new(Axis::z().reversed(), 2.0)`
are one surface holding different numbers in three of nine slots. A cone is
double-napped and a bend's centre cannot move, so turning either round is the
same point set too. **A §16.10 negative claim, carried as a paragraph, inside the
function whose whole job was this class** — the seventh sentence of §51.4's shape
and the one that had been load-bearing longest.

**Turning a stored axis round moves no corner, and it is measured.** 72 curved
surfaces against 153 plane pairs plus 36 same-bend curved pairs against 18
planes, every subset of the axes turned round: **12 960 comparisons, 7 584 of
them placing corners, 0 differences** — same points, same order, bit for bit,
refusals matching refusals. The instrument is live: swapping the reversal for an
origin *slide* of 0.1 gives **587 of 12 960 differing**.

**It renumbers no `which`, for a structural reason.** `which` counts along the
line the **two planes** cross in; `how_it_sorts_against` compares the kind first
and two of three must be flat to be solved at all, so a curved surface is always
the third row and the line is fixed by the other two. `three_planes_faced_the_one_way`'s
stated reason for handing curved triples over untouched — *"turning a plane round
turns that line round"* — is about a **plane**, and does not carry to an axis.

**The fold went in the geometry key and nowhere else.**
`name_of_the_surface_either_way_round` and `names_of_the_three_surfaces` fold the
axis direction the way they fold a plane's facing. **Not**
`how_it_sorts_against` or `name_of_surface_bits`: their job is a total order on
stored numbers, folding there makes two distinct bit patterns compare *equal*,
and `canonical_cut_surfaces` then dedups on a tie broken by whichever face handed
its cuts in first — reintroducing the order dependence the fixed order exists to
remove. And no curved analogue of `three_planes_faced_the_one_way`, because the
sweep says the solve was already neutral and there is nothing for one to do.

**No fuse-level test passes, and the wall is demonstrated rather than asserted.**
Two spellings of one pipe need two solids carrying that pipe, i.e. coincident
cylindrical faces, and the fuse refuses that first — `Undecidable`, *a point
inside a piece of a face*, enclosure `inf`, identically for one spelling and for
two, found independently by both lanes. So the fuse-level row is a **pin in
§51.1's shape**: it asserts the refusal is the same whichever way the pipe is
written, and fails the day the wall moves.

**A fifth spelling freedom does not exist**, and that is counted rather than
asserted: stored numbers against real degrees of freedom are plane 4/3, ball 4/4,
pipe 7/5, cone 7/6, torus 8/7, and every leftover is the direction's length, its
sign, or the pipe's slide. A negative radius or a negative slope is refused, not
folded; a torus with its radii swapped is a different surface; a cone's apex and
a torus's centre are determined points. **Why the slide is a pipe's alone**: a
plane stores one offset number where a pipe stores a whole three-number origin of
which only the part across the axis matters, so two in-plane points either round
to one offset or make two planes a rounding apart — never two names for one.

### 51.10 The cheap form of a gate: a reader with a reason to be adversarial

§51.8 records that a closing gate which re-derives every load-bearing number
beats one that reads the lane's report. This unit found the cheap form of the
same thing by accident, at the very end, and it is worth having because a gate
costs a lane and this costs a sentence.

**What happened.** A downstream consumer was told, in conversation, that a full
circle is walked in *at least four steps* however coarse the deflection is —
true, and `by_turning`'s quarter-turn cap is why. They restated it as *at least
four points* and said they were about to turn it into a compile-time assertion.
That sentence is wrong: `walk.rs:349` is `let points = (1..steps)`, both ends
excluded because the ends are the corners, so `steps` steps emit `steps − 1`
points and the floor is **three**. It was caught in the minute between their
saying they would pin it and their pinning it.

**A correct number in the wrong unit**, which is §51.4's family with a shorter
fuse: *at least four* was true of steps and silent about points, and the two
differ by exactly the ends-excluded rule both sides had already written down.

**The active ingredient is not the test.** Nobody re-read the sentence more
carefully. What happened is that the claim acquired **a reader with a reason to
be adversarial about it** — being about to be pinned made it worth opening the
emit line rather than trusting the recollection. A test is a reliable way to
manufacture that reader on a schedule; the announcement of one manufactures it
for free. So, alongside §51.6's rule:

> **Telling the owner of a fact that you are about to depend on it is most of a
> gate at none of the cost.** The moment to do it is when the claim is about to
> become load-bearing, because that is when being wrong is expensive and
> therefore when somebody will actually look.

**And pin the mechanism, not the arithmetic.** The fix was not to pin `>= 3`.
A pinned point count moves for two different reasons — the cap changing, or the
emit arithmetic changing — and a reader hitting the failure cannot tell which.
Pinned as *a closed run is cut into at least four segments*, a failure means the
quarter-turn cap or the fewest-steps floor moved, which is a sentence somebody
can act on. Correct and useless is a real category.

**One argument for not copying a fact across a boundary**, better than the usual
one. The usual reason is that two copies can disagree. The stronger reason is
that a copy restated in the copier's own words can be **wrong and internally
consistent**, and then it passes — where two copies that disagree are the case
you find out about. Facts rcad measures are asserted in rcad's suite and cited,
never restated, by anything downstream.

