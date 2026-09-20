## 49. Nothing round was ever checked, closed — recorded 2026-09-04

56 commits, 22 files, +7352/−344. **Thirteen lanes, one spike and two closing
gates**, against a budget of 9 agent-hours with 2.0 reserved for the gates before
the first lane started. The suite goes from **1508 to 1579 tests, none skipped**.

### 49.1 The sentence was not earned, and the gate is why we know

§48.3 said:

> Every algebraic law in `fuses_that_must_agree.rs` runs over a round solid, and
> transform invariance runs over one first.

**The first clause is false at the closing tree.** The number gate listed the
properties and their fixtures rather than reading the doc:

| | laws |
|---|---|
| **run and compare** over a round solid | 4 — commutativity (bar, ball, frustum), inclusion–exclusion (ball), idempotence and `A \ A = ∅` (ball, bar), transform invariance (bar, ball) |
| **have a round fixture and compare nothing**, pinned at a refusal | 4 — absorption, `A \ (A \ B) = A ∩ B`, `((A ∪ B) \ B) ∪ B`, associativity over the trio |
| **have no round fixture at all** | 3 — the two multi-argument chains and the repeated-union property |

Curved fixture sites went **5 → 19** and laws comparing over a round solid
**1 → 4**. That is a large move and it is not *every*.

**The second clause is true and fully earned.** Transform invariance runs over a
round solid bit for bit with no tolerance introduced; `moved_by` handles all five
surface kinds and the panic that stood in its way is gone. Its one weakening — the
round rows shift and never turn — was checked against the public API by the gate
and is a **limit**: `Isometry` has exactly one turn-bearing constructor, and
`cos(π/2)` in `f64` is not zero.

**The honest sentence, and it is the one this unit earned:**

> **Four of the eleven algebraic laws now run over a round solid where one did,
> transform invariance is the first of them, and the seven that do not are each
> pinned at a named refusal rather than passing green.**

### 49.2 The unit's largest result is a wrong answer it introduced and caught

**A fuse ordered a closed quartic loop as an endless straight line.** Two
cylinders whose stored axis directions are *not* parallel — exact cross
`5.1e-17` — but whose `f64` cross rounds to exactly zero, because both products
round to `0x1.7271392ba6f36p-2`. The straight gate said parallel, `intersect`
said `RightRoundWhereItsFacesCross`, and `how_a_crossing_runs` returned
`AlongAnAxis`.

Yesterday that pair **refused**. A lane closing a real gap — two overlapping
parallel pipes were refusing for no arithmetic reason, because the two doors
disagreed — carried a rounded test into a place where it decides an answer
instead of a refusal. **The trade this kernel exists not to make, made by a lane
fixing something else, and reachable from file data** (`rcad-step`'s reader
builds a `Cylinder` straight from the file).

The closing claim gate found it. Not a lane, not the number gate, and not the
dispatcher, who had verified the lane's own claims and merged it.

**And the fix found a bigger one underneath.** The plane-against-pipe row of the
same two doors had the identical defect and it is **older than this unit**: of
45 186 near-square planes drawn against random pipe axes, **9 239 — one in five —
have a stored normal whose `f64` dot with the axis is exactly zero and whose
exact dot is not**, and `intersect` names every one of those crossings a closed
ellipse. Each was being ordered along an axis. Both rows are now three-way
predicates — `Pos`, `Zero`, or `Undecidable` carrying its enclosure — asked by
both doors, in the style `along_a_crossing.rs` already uses.

**A third rounded row is still rounded.** `a_plane_cuts_a_cone_in_rulings`
decides a degree-six expression in plain `f64`, and its doc still claims that
disagreeing with `intersect` costs a refusal rather than a wrong answer. By the
two findings above that is likely false in the same way. **It is not
demonstrated**, and the lane that found the other two said so instead of
assuming. It wants its own lane.

> **§49.8 closes this: it was demonstrated, and it was a bug.** The paragraph
> above stands as written because it was right to refuse to claim it — but the
> claim it declined to make was true, and the row is now three of three.

### 49.3 What else the two gates found that thirteen lanes did not

**Option D fires.** §42.9's fallback has been carried for three units with its
condition never evaluated; §44.6 wrote the condition so it could fire and scored
it at 50–56%, "exactly on its own boundary"; §47.3 promised to score it and did
not; §48.2 item 3 sent it to this gate. Scored on §44.6's own denominator: 323
KiCad files, 226 refused, 73 accounted for by named gaps still open, leaving
**153 of which 94 miss by more than `f64` rounding — 61.4%**. Both corpora,
50.8%. It is clear of its boundary, and the numerator can only grow with better
instrumentation, so the verdict is one-sided safe.

**§48.9's prediction landed inside this unit's own tree.** The frustum family's
comparison floor is justified by *"eighteen fresh-seed runs came out between 30%
and 46% … a margin of ten points."* The gate ran 24 seeds and measured **25.4% to
52.4%**. The decision to lower the floor is *better* supported than its own
evidence — the old floor would have failed on the gate's first run — but the
sentence beside it is false and the margin is **5 points, not ten**. A number
that was right with a sentence about it that was not, which is exactly what
§47.8 predicted and §15.10 was written to catch.

**Seventeen published sentences were false or overstated**, against §47.4's five
in the unit before. Three deserve naming because of where they were:

1. `Measurement::volume_error` and `Measurement::area_error` — **public API
   contracts** — both promised the mesh never overestimates. A cube of side 8
   with a radius-1 hole measures **0.477 too much** at deflection 0.1; an 8×8
   plate half a unit thick with the same hole measures **0.127 too much** skin
   while the same hole eight deep measures 0.128 too little. The magnitude bound
   survives two-sided; the sign was the lie.
2. `ResultIsNotManifold { count: 1 }` was recorded as **withdrawn** in §38.6,
   repeated in §39.4, and told to every future lane by the `fuse-debugging`
   skill. It is **live**: 83 268 fuses reach it in two of six families, and its
   sharpest form has no curve and no rounding in it — three axis-aligned boxes on
   the integer grid where `fuse([far, near], [bar], Intersection)` refuses while
   `near ∩ bar` alone answers with volume 2 and validates. **The argument that
   contributes nothing turns an answer into a refusal.** The four families the
   old record was written from reach it in none of 3 000 fuses each, which is why
   it looked closed.
3. A lane corrected a sentence about cone crossings in `assemble.rs` and left the
   identical sentence standing in `arrange.rs`. **The tree now carries the
   corrected and uncorrected forms of one sentence, two files apart** — this
   unit's own recurring finding, happening to this unit.

**Two latent defects with named one-line fixes**, neither exercised today:
`settle_on_the_lower_rounding` never removes the superseded key, so a coordinate
no corner sits on can merge a later differently-named corner into an old one; and
`three_planes_faced_the_one_way` canonicalises to `-0.0` where its sibling adds
`+ 0.0` to avoid exactly that, and `total_cmp` sorts `-0.0 < 0.0`.

### 49.4 What was fixed, and what each cost to find

| what | was |
|---|---|
| a fuse depending on which way round its arguments were handed over | **on axis-aligned whole-number input** — §17.7's control family. Fixed in three lines: a solid's own planes now cut its own faces. Volume 10 and area 38 against hand arithmetic, not against the other order |
| one corner named twice | two solids flush on a plane hold it facing opposite ways, and `CornerName::Meet` keyed on the stored coefficients, so `P` and `−P` were two names. `name_of_the_surface_either_way_round` already existed and was wired into two of its three call sites |
| two parallel pipes refusing for no arithmetic reason | the two doors disagreed; one had a real test and the other swept the pair into a blanket `false` |
| one plane and two quadrics refusing on sight | where two quadrics share a quadratic form, their difference is **linear** — so the triple was two planes and a quadric all along, and needed no quartic |
| a fuse giving up over a triple that provably has no corner | `intersect` is now asked about the two partners, and only `SurfaceMeet::Nothing` skips. Pinned in both directions: never-skip fails 4 tests, always-skip fails exactly 2 |
| a mesh's measured volume with no published bound | `skin · d · (R/(R−d))²`, infinite unless `d < R`, worst ratio 0.1999 and loosest 0.00446, with the exponent fitted over 60 points: the error goes as `d¹`, and `d²` under-predicts the finest point by **124×** |

### 49.5 What was learned about running the work

- **The gates earned their reserved budget twice over.** They found the
  regression, Option D's verdict, §48.3's falsity and the frustum floor's false
  margin — none of which any of thirteen lanes reported, and one of which the
  dispatcher had already verified and merged.
- **§48.4's stated risk was a spelling, not a fact.** The spike was written to
  test whether a triple product that is zero as a polynomial is decidable in
  `f64`. Kept inside `Exact` as a determinant it is exactly `Decision::Zero`;
  built as three `f64`s first it is 4.65e-17 against a signal of 9.95 — which
  would also have passed, and is luck rather than cancellation. **The same
  distinction is the whole of §49.2.** The unit learned it in a 20-minute spike
  and then shipped a bug that is the same mistake in a different file.
- **A lane told to justify a sentence beat a lane told to check code, again.**
  Both gates ran on the same tree for a comparable budget; the claim gate found
  the wrong answer.
- **Two lanes improved on their briefs by contradicting them.** One was told a
  seamless cylinder wall is what `ACurvedFaceGoesRightRoundWithNoSeam` exists to
  refuse — it is not, that refusal is gated on a surface that *pinches*, and a
  cylinder does not. One was told to expect a frustum family at 9 of 96 and found
  two fifths of it answering, then found the rule that says which two fifths and
  pinned it at 84 of 84.
- **Three of thirteen lanes patched a `.rs` file with a heredoc or `sed -i`**,
  which CLAUDE.md bans categorically, and all three reported themselves. A rule
  broken by a fifth of the lanes that had read it is a rule the tooling does not
  enforce. Recorded rather than enforced, by decision.
- **A lane's brief that named what would count as failing produced better work
  than one that named what to build**, for the third unit running.

### 49.6 What stands

| open | where |
|---|---|
| ~~**the third rounded row**~~ | **closed by §49.8.** It was a bug, and it needed both of its two signs to be wrong at once |
| **ordering along an endless bend** | the spike passed: `sign((p₁ − p₂) · (n × own_way))`, 16 400 pairs, none undecided, none out of order. One `HowItRuns` variant and the call sites. **X2 measured what it buys**: the frustum family answers iff `slope · (roof − apex) < distance to the nearest side wall`, 84 of 84, so this is the other three fifths |
| **`CannotTellTheCellsOfAFaceApart`** | no longer undemonstrated. A face with a round hole has a rim wire of one closed edge, and a one-corner wire has no winding to read. Blocks associativity over a curved trio |
| **the general quartic** | plane and two quadrics that bend differently. The pencil route wants `det(λM₁ + μM₂) = 0`, whose cubic roots are not exact, so every sign below them is a box |
| **`ResultIsNotManifold` with `count` other than 1** | entirely unmeasured — 864 of 31 104 in an exhaustive pair sweep, counted by nothing |
| **`select` on a curved cap** | two overlapping parallel bars now get past geometry and stop here, undecided with an unbounded enclosure |
| **the two latent corner defects** | §49.3, both with one-line fixes named |
| **`cuts.rs` and most of `three_surfaces.rs`** | **unaudited**, and recorded as unaudited rather than as clean — the claim gate audited by targeted attack and said so |
| **the writer, the torus, cone against pipe** | unchanged from §47.7 |

### 49.7 The prediction

§48.9 predicted this unit would lose to a row in an open table that is *true but
too small*, and it did — the frustum floor's "30% to 46%" was measured over
eighteen runs and is 25.4% to 52.4% over twenty-four. The countermeasure held:
the gate caught it because §15.10 forbids accepting a moved number without its
case list.

**So the prediction for the next unit is about what got through instead.** The
regression in §49.2 was introduced by a lane whose every claim was true, verified
by a dispatcher who checked each of them, and caught only because a gate went
looking for sentences rather than numbers. Nothing in the number gate's ten rows
would have found it, because **the defect was not in any number anybody
published** — it was in an expression nobody had asked to be exact.

> **The next unit will ship a correct fix that carries a rounded test into a
> place where it decides an answer.** The countermeasure is the one §49.5 names:
> when a change moves a test from a place that refuses to a place that answers,
> the test itself has to be proved and not merely reused. Score it by asking, at
> the next gate, whether any predicate reached a new call site without being
> re-derived there.

### 49.8 The third row was a bug too, and three of three is a class

Run after §49 was written, on the row §49.6 listed first among what stands.

**The dangerous direction is the same one**: the gate says *straight*, the
crossing is really a conic, and `how_a_crossing_runs` returns `AlongAnAxis` —
**a closed ellipse ordered along an axis**, exactly as the cylinders were.

What hid it is that it takes **both** of the function's two signs being wrong at
once, which is why nobody found it by inspection:

| the gate said | the truth | draws | what happened |
|---|---|---:|---|
| rulings | one closed ellipse | **3 655** | `AlongAnAxis`, all 3 655. **Not one refused** |
| rulings | the apex alone | 16 077 | no run exists to order |
| a curve | two rulings | 7 761 | harmless **only** because `intersect` refuses to name a near-tangent ruling direction |

Out of 360 000 draws over three families. Either sign wrong alone is safe: a
rounded apex with `E < 0` makes the truth a hyperbola, and `intersect` refused
all **17 645** of those because a branch clearance of about `1e-17` cannot beat
one ulp of `n·a`; a rounded lean with the apex genuinely on the plane makes the
truth `TouchAtOnePoint`, and no run exists. Both wrong together names one closed
ellipse and calls it two rulings.

Both signs are now exact `Decision` predicates asked by both doors, with the lean
written as `(n·w)² − Σ(slope·(n_j w_k − n_k w_j))²` so it needs only squares of
short expansions and never a product of two long ones. The nearly-tangent plane
that used to be misordered is now **ordered correctly** — `RoundAClosedCurve` —
which is a better landing than the plane-and-pipe row got, because a cone's
ellipse is not `1/(n·w)` long and a place to start on it is still findable. Every
cone rate is bit-identical, including the frustum-against-a-block row at 90 of 96.

**The reuse was not achieved and that is the open item.** `intersect` computes
both signs exactly already, inside `where_a_plane_and_a_cone_cross`, over an
`Exact` that is `pub(crate)`. Asking one question in one place needs two
functions exported from `rcad-geom`, which was outside the lane's write set — so
the expression now exists in a third copy, guarded by a 360 000-draw test
asserting the door and `intersect` agree on every draw. **That guard is not the
same thing as not having a copy**, and §49.5's lesson says so.

> **Closed.** The copy is gone. Both signs are built once, in
> `intersect.rs`'s `where_the_apex_stands` and `how_the_plane_leans`, and asked
> two ways: `where_a_plane_and_a_cone_cross` puts them through the policy, and
> the two exported functions `where_a_cones_apex_stands_against_a_plane` and
> `how_a_plane_leans_against_a_cone` stop at the cascade, which is what both
> doors call. **The doors take no `Policy` on purpose** — a tolerant one may read
> an enclosure the cascade refused as nothing, and a `Zero` a door has not earned
> is what calls a closed ellipse two rulings. The two therefore part company only
> under a tolerant policy and only into a refusal, which is the trade
> `a_plane_lies_along_a_pipes_axis` already states one row down. The 360 000
> draws stay: with one implementation they no longer compare two spellings of the
> arithmetic, and what they check now is the *table* — this pair of signs means
> straight, that pair means curved — against the shape `intersect` names, which
> is still written out in both crates. Every rate is bit-identical, the frustum
> row included.

**Three rounded gates, three bugs, all the same shape**: an `f64` expression
compared against zero, believed to fail into a refusal, in fact failing into a
wrong answer. That is not three incidents. The next unit should ask what *else*
in `arrange.rs` and `assemble.rs` decides a shape by comparing a sum of products
to zero, and treat a rounded answer there as a defect until it is shown
otherwise — the reverse of the burden these three carried.

Two paths remain unexercised and are recorded as such rather than as clean: the
`None`/`Undecidable` arm of these predicates never fired in 360 000 draws or
anywhere in the suite, and neither did its twin next door; and the 7 761 draws of
the safe direction are safe only until
`TwoLinesThroughAConesApexAreOneCurveThroughItself`'s neighbouring gap closes.

