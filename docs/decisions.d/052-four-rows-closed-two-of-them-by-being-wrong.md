## 52. Four rows closed, two of them by being wrong, recorded 2026-09-05

*Six lanes and two gates against §51.7's four defect rows. `just default` green
twice at the end, 1698 tests. Written from the gates' verdicts rather than the
lane reports; where a gate and a lane disagreed, the one that re-derived from the
tree wins. Every number below names the command or test that produced it, which
is §51.6's own rule applied to a record and not only to a plan.*

### 52.1 Two of the four rows were wrong about themselves

| row | what it said | what it was |
|---|---|---|
| `room_left_over` records nothing | a defect, plus *a caller cannot tell* | defect real, **second clause false** |
| an empty walk on a closed edge | unknown behaviour | **not a defect** — refused twice over |
| the pointed cone refuses coarse | a refinement-geometry accident | **a §16.10 gap in the bound**, fixed |
| an oblique plane cutting a cylinder | a gap, past 60° of tilt | **false, and backwards** |

That is the finding to carry forward. §51.7 was written by a unit that had just
audited itself carefully, and half its defect rows still misdescribed the defect
they named — not by being vague, but by being specific and wrong. Every one of
the four was **stated with a mechanism**, and the two that were wrong were wrong
about the mechanism while being right that something was there, or right that
nothing was there for the wrong reason.

### 52.2 Row 1 — a wave-through now leaves a record

`room_left_over` (`walk.rs`) hands a tolerant caller an empty point list when the
crossing's own rounding eats the whole deflection. Two of its three sibling sites
carry a signal and it carried none. It now takes a `log: &mut ProvenanceLog` and
pushes one record on that path.

**The threading never leaves the file**, which is what made this the small lane it
looked like: `walk` already had a log, and `walk_an_endless_bend` — which did not
— is called only from `walk`, twice.

The record follows the precedent already in the crate, `DynamicRange::check_under_policy`
(`scale.rs:243-261`), which records a check that is not a predicate the same way:

```rust
Record::new(ROOM_LEFT_TO_REFINE_A_WALK, &[], Level::Float,
            Decision::Undecidable { enclosure: deviation }, Measures::Length)
```

`Measures::Length` makes `record.wobble(ws) == deviation` for every working size —
`dimension = 1` makes the fold at `policy.rs:241` the identity — so this record
cannot turn an accepted walk into a refused solid: the same number just passed
`policy.wobble_is_acceptable` at the push site.

**The stronger form of that, which a gate established and the lane did not:** a
fuse never sees one of these at all. `fuse/mod.rs:183` is the only line outside
tests that compares a log's `worst_wobble` against a budget, and instrumenting it
to record any `room left to refine a walk` it met produced no file across all 1698
tests. `walk` is reachable only from `mesh.rs` and from `validate.rs`'s
`the_ring_a_wire_traces`.

`crates/rcad-geom/tests/a_waved_through_walk_leaves_a_record.rs` fails 2 of 4 on
`22a163a` and passes 4 of 4 here. Its other two are anti-vacuity halves and they
are load-bearing: moving the `log.push` to fire unconditionally fails both.

### 52.3 The clause that was false, and why its replacement needs three exits

§51.7 said the empty list *"a caller cannot tell from a stretch that legitimately
needed no interior points"*. **It could.** `FEWEST_STEPS_THAT_SHOW_A_BEND`
(`walk.rs:930`) is 2 and both step counts floor at it, so neither emitting range
can come out empty; the only other empty-points exit carries a deviation of
exactly `0.0`; and on a wave-through the deviation is at least the deflection,
which `Deflection::of` refuses to let be zero. So
`points().is_empty() && deviation().magnitude() > 0.0` already *was* the
wave-through, and the fix is the log record alone.

**That argument has three exits from `walk` and not two**, and the third is the
one a reader will miss. `follow_the_quartic` (`walk.rs:666`) returns a positive
deviation and never calls `room_left_over`, so it never pushes a record. The
claim survives only because `march` cannot come back empty: on its first
iteration `travelled = 0.0`, so the early return is false and a point is pushed
before any exit can be taken. A gate ran this as a live invariant over the whole
suite — asserting the equivalence on every `Ok` return — and it never fired.

### 52.4 Row 2 — refused twice over, and the answer is scoped

An empty walk list on a **closed** edge never reaches a consumer, because
`SolidView::mesh` refuses the whole mesh and `points_along_edge` is never
published for that solid at that deflection.

**Two independent barriers, demonstrated rather than read.** A closed edge is the
only edge of its cap's wire, so that loop has one corner and `Loop::through`
refuses it — `TooFewCorners { count: 1 }`. The wall's wire is *not* one corner, so
that check says nothing about it, and it is refused separately by
`AWireWillNotLieFlatOnItsFace`. Independence was shown by loosening
`Loop::through` (`domain.rs:27`) from `< 3` to `< 1`, watching the cap through,
and catching the wall underneath — done once by the lane and again from scratch
by a gate.

The shape it takes is the finding. The deflection floor and the crossing's own
rounding are deliberately in step for an ordinary bar, so a closed edge cannot be
starved on one. They come apart when a **cap plane leans hard**: a bar of radius
1 with its top cap normal `(2e9, 0, 1)`, at `Policy::tolerant(1e11)` and
deflection `5e-4`, has two closed edges that answer differently — the flat bottom
circle walks into points, the leaning top's ellipse comes back with none.

**Scope, and it matters downstream.** What is demonstrated is *no route the suite
can build*, not *never*. A face whose wire held the closed edge **alongside other
coedges** all meeting at that vertex would contribute one corner instead of *n*
and could still clear the three-corner rule — and then a `Mesh` holding an empty
list for a closed edge would exist. Nobody reached the question of whether a
builder or the deserialize door can make such a wire. §52.11 keeps it.

### 52.5 Row 3 — the bound charged a ruling

The pointed cone refused at 1e-3 of its own size with finer deflections meshing
either side. Reproduced exactly on `22a163a`: **6 of 21 deflections refused, 9
finer ones meshed anyway, the ladder changes its mind 6 times**
(`cargo nextest run -p rcad-topo --test merged -E
'test(~whether_a_coarser_deflection_always_meshes::the_pointed_cones_refusals)' --no-capture`). §51.7's numbers were inherited
correctly, which is worth saying because most of this section is about ones that
were not.

**A §16.10 gap, and the answer was one step from the end of a derivation already
written down.** `how_far_a_flat_piece_of_a_cone_strays` goes
`Σλᵢrᵢ(1 − cos(uᵢ − u_k)) ≤ maxᵢ rᵢ(1 − cos Δ)` and then bounds every corner's
radius by the patch's largest, collapsing it into one chord. **A corner at the
apex has `rᵢ = 0`**, and a triangle with two corners on the apex is a *ruling* —
in 3D it is the segment from the apex to its third corner, lying exactly on the
cone and straying nothing. The old bound charged it `0.7071·v`. The fix keeps
each corner's own radius.

The refusal was `RanOutOfRoomBeforeMeetingTheDeflection` from `refine_the_inside`,
with the packing floor `step/8` declining both wanted midpoints. It is not the
`√d`-against-`d` crossing the fixing lane wrote down — see §52.8.

**What actually moved** (1e-4 of size, the 140-line family sweep diffed between
the two trees): the five cone and frustum solids, `53742→32812`, `56880→14550`,
`31666→29144`, `71648→724`, `77874→1026`. The other fifteen solids are
line-for-line identical at every rung.

The pointed cone now meshes every rung from `0.2` (44 triangles) to `1e-5` (6488)
and is stopped instead by how many points a patch will hold —
`MoreTrianglesThanAPatchWillHold` from about `5.96e-6` down, `would_need`
6 282 880 at 1e-6. That limit is **monotone**, and provably so:
`a_step_that_stays_within` builds its step from `radius` and `share`, neither of
which depends on the deflection, and `refuse_a_patch_too_big_to_hold` runs before
a single point is placed. Once it says no it says no all the way down.

### 52.6 Row 4 is backwards

§51.7 expected `BoundarySideStraysFurtherThanTheDeflection` past roughly 60° of
tilt where a plane cuts a cylinder. **It never fires**, at any tilt, on either of
two probes built through the public API — a bar cut by a leaning plane, and the
shape the row itself names, a bar fused against a turned cube, swept to 89.5°.

The measurement inverts the row. At `r = 0.05`, deflection `5e-4`, tilt 89.4°, the
wall's 570 boundary sides split into 502 on the tilted ellipse at `0.005222` of
the deflection and 52 on the **flat bottom circle** at `0.486075` — the tilt
making a side **cheaper by 93×**. The worst consumer of the boundary budget on
that solid, at every tilt, is a plain un-tilted circle.

**The expected one-line fix was already in the tree**, and this was settled by a
prediction rather than by reading. `how_many_steps` was hand-computed for both
candidate reaches: `Δu = 1.251631e-2` for the ellipse's own `r/cos θ` against
`1.208305e-1` for the cylinder's `r`. Observed: `1.25163e-2` on the ellipse's
sides and `1.2083e-1` on the bottom circle's, where the reach genuinely *is* `r`.
Both candidates visible in one run, every digit matching. `Conic::how_far_it_reaches`
is `across.length().max(and.length())` and `where_a_plane_cuts_a_pipe` stores the
semi-major in `and`, so the walk has always stepped against the ellipse's reach.

**The negative claim, earned properly on the second attempt.** The first attempt
argued that a smaller reach could not cross the threshold, which is backwards — a
smaller reach gives a finer step and a *smaller* stray. The real argument is the
budget split: `WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO` is `3/16` of the deflection
(`mesh.rs:2117`) and the constant's own doc derives that the triangulator reads at
most `8/3` of what the walk was asked for, so `strays ≤ (8/3)(3/16)·deflection =
deflection/2` for **any conic, any reach, any tilt**. Demonstrated over 747
configurations and 159 110 boundary sides: nothing fired, worst `0.498389` on a
cylinder and `0.49347` on a sphere — 99.7% of the ceiling, which is what makes it
a demonstration and not a paragraph. Scoped to the solids `build.rs` builds and
fuses of them; a cone or a torus is untested and `Model::builder().add_face` is
public.

### 52.7 A test that could not see the bound it measured

**The unit's largest finding, and no row named it.** `triangulate.rs` carries a
§15.6 search for how much of the cone's published bound a real triangle uses. It
reported `0.39447768211554846` after the fix against `0.39447768211554857`
before — **one ulp, after a change that cut a frustum's mesh from 56 880
triangles to 14 550.**

Not luck. The search's family was `[[0, far], [span, far], [span/2, near]]` — two
corners on the same widest ring — and for that family the collapsed bound and the
per-corner bound are *algebraically identical*, both `2·t·far·sin²(span/2)`. All
240 triangles evaluated the same number under both formulas. The search was
measuring a bound the code no longer computed, and the doc comment above it still
stated the old claim word for word.

The replacement family is 12 rows × 8 slopes × 6 spans = **576 triangles**, and it
carries its own proof that it can tell the two apart —
`what_the_collapsed_bound_would_have_charged` runs beside the real bound on every
one:

| shape | new ÷ collapsed |
|---|---|
| the six old `[far, far, near]` rows | exactly **1.0** — the blindness itself |
| frustum bands 1→2, 1→3, 2→8 | 0.5, 0.3333, 0.25 — that is `near/far` |
| a corner on the apex | 0.125 – 0.197 |
| the 48 rulings | **0** — bound is slop, collapsed charges the whole span |

Worst share on the new family is `0.6589159410563914`. Putting the collapsed
radius back fails the search rather than moving its number.

**The general shape:** a published bound was tightened and its own §15.6 search
could not see the tightening, because the search's family sat exactly on the
subspace where the two formulas agree. A test can be blind to the thing it exists
to watch, and green is what that looks like.

### 52.8 Eight false sentences, and where each sat

§51.4's shape for the fifth consecutive unit. The new column is *who could have
caught it*, because that is what changed this time.

| the sentence | what was true | caught by |
|---|---|---|
| §51.7: a caller cannot tell an empty walk from a stretch needing no points | it could; §52.3 | the lane, checking its own brief |
| §51.7: the ellipse's `r/cos θ` outruns the deflection past 60° | backwards — the tilt makes a side cheaper by `cos θ` | a spike told to distrust the row |
| §51.7: refining against the ellipse's own reach is a one-line change | already in the tree since `Conic::how_far_it_reaches` was written | the same spike |
| a lane: **no mesh anywhere got coarser** | meshes of cones got much coarser — 3.9× on a frustum with no apex, which was not the bug. The claim that survives is that the bound is still an *upper* bound | a gate diffing the family sweep |
| a lane: the ragged edge is the packing floor's `√d` crossing the height's `d` | that predicts a monotone floor; the data is a **band**. The strays ladder is `1.4142/2^k`, the same at `d = 2e-4` and `2.518e-4` | a gate instrumenting the loop |
| a lane: B's leaning cap is *the same one-line change in what `walk` is handed* | three quantities, not two; §52.9 | two gates, from opposite sides |
| a spike: neither probe refuses | one refuses in 8 of 20 cells, in the **fuse**, over the whole 30–70° band the row blamed. The conclusion survives because the other probe covers that band; the sentence does not | a gate re-running the probes |
| a spike: eight boundary sides it could not explain | **sixteen**, and they are `cut_in_two` cuts rather than walk steps — the mismatch between two independently walked chains, bounded by their half-sum `6.67e-2` against `6.3544e-2` observed | a gate told to go and find out |

Seven of the eight sit **beside a number that was right**. That is now four units
running, and the countermeasure that worked here is not a better sentence — it is
that somebody re-ran the thing the sentence was about.

### 52.9 §51.6's prediction, scored: fired, on a surface it did not name

> *The next unit's largest error will be inherited from a section it quotes, not
> produced by a lane.*

**Fired, and the scoring is not close.** The two rows that were wrong about
themselves were both inherited from §51.7 verbatim, and one lane inherited
§51.7's proposed fix into its own handback without checking it. The counter-example
is that two of the eight false sentences were lane-produced.

**But the largest error was not a number at all, and that is the new part.** The
blind search in §52.7 is an inherited *test*: the doc comment stated a claim from
before the fix, and the fixing lane changed the code under it without opening what
its test measured. §51.6's countermeasure — *name the test or the command that
produced a number* — would not have caught it, because the number was produced by
exactly the test that was named, and the test was wrong. So:

> **When a fix changes what a published bound computes, the bound's own §15.6
> search must be shown to move.** A search whose number does not change is not
> reassurance; it is the first thing to distrust. Show it failing under the old
> formula, the way a test is shown failing without its fix.

**And the cheap gate of §51.10 paid again, twice.** A peer session independently
derived the row-4 correction and sent it over; it arrived after the spike had
already demonstrated it, which turned an agreement into a confirmation. The other
direction paid too — telling that session which run confirmed its facts, rather
than that it was right.

### 52.10 The leaning cap, and a §19 nerve

Sized by a gate rather than named, because the row it came from was about
something else. The mesher's deflection floor
(`how_far_a_mesh_point_of_this_face_rounds`, `mesh.rs`) is built from a face's
**corners** plus the surface's radius. For a leaning cap that under-charges, and
the measurement is stark: the floor is **flat at `6.1432e-13` at every tilt**,
while the walk's own demand is `256·ε·(lean + 0.5)`. The walk crosses the floor at
a lean of about **10.8**, and by `lean = 2e9` the floor is claiming nine orders of
magnitude of deflection workable that are not — off by `1.85e8×`.

**The runaway term is not the one anybody named.** It is not the ellipse's
semi-major. `Conic::size` is `how_big_this_is([middle, axis.origin()], …)` where
`middle` is the ellipse's **centre**, so it grows with *where the conic sits* up
the pipe's axis. Measured `closest` matches `256ε·(lean + 0.5)` and not
`256ε·√(lean² + 1)`.

That is a §19 nerve, and the code says so itself.
`Conic::how_far_it_reaches`'s doc comment reads *"Where a circle sits is not
allowed to change how finely it is cut (§19)"* — and today where it sits does
change how far off it the kernel says its points are. A **gap**: the answer
exists, and the floor cannot see it.

### 52.11 What stands

| open | where |
|---|---|
| **the leaning cap under-charges the deflection floor** | §52.10. A §19 nerve with a number: crosses at lean ≈ 10.8, `1.85e8×` at `2e9`, and the term is the conic's stored `size` rather than its reach |
| **the wave-through record says `Undecidable` for an event that chose to go ahead** | nothing was undecided; a policy accepted a wobble. `Cause::TolerantFallback` is the honest cause and `Record::resolve_to` is `pub(crate)` to `rcad-predicates`, so `rcad-geom` cannot write it. A **gap in the API**, not a design choice, and it has two consumers already — this site and `scale.rs:243-261`. The constructor asked for is `Record::an_operation_settled(operation, inputs, approximation, measures)`, which adds no `Cause` variant and so pays none of §25.8's renumbering cost |
| **a wire holding a closed edge alongside other coedges** | §52.4's unreached route, and the one way an empty list could still reach a `Mesh`. Nobody established whether a builder or the deserialize door can make one |
| **the `crowded`-only route to `RanOutOfRoomBeforeMeetingTheDeflection`** | reached by nothing. Tallied over the workspace: that refusal came back **once in 1694 tests**, with `crowded = 0, declined = 2`. Recorded on the variant's own doc |
| ~~**`Mesh::triangles()` publishes two claims and guards one**~~ | **closed.** `mesh.rs:61-70` promises *faces in the solid's stored order* **and** run-to-run stability; only the second bit, because the nearest test collected `triangle.face()` into a **`HashSet`** (`meshing_a_curved_solid.rs:728-735`) which passes under any permutation, and `measure.rs:662` needs only that the order is *fixed*, not that it is the stored one — so a stable but different order kept every sum bit for bit and broke the promise silently. Both halves hold, and are now pinned as **two separately-named tests** in `what_order_the_mesher_hands_its_triangles_back_in.rs`, each shown failing under its own mutation: permuting whole face-blocks breaks stored order and leaves contiguity intact, which is what proves the two are independent. Raised by a consumer announcing it was about to depend on it — §51.10's third payment, again catching rcad's own gap |
| **a test that walks the same iterator as the thing it tests** | not a defect, an averted one, and worth the row because it is §52.13's shape one level up. `Mesh` emits in `SolidView::faces()` order and "the solid's stored order" *is* `SolidView::faces()` — so a test comparing the two would have moved with the mesher and stayed green while the promise broke. The pinning tests therefore read arrival order off the **geometry** instead: a bar's blocks must be a plane facing `[0,0,-1]`, a plane facing `[0,0,1]`, then a face that is no plane at all; a cube's must be its six exact outward normals. Exact `f64`, no tolerance. **A test must not derive its expectation from the machinery under test**, and the cheap check is whether the expected value would change if the code did |
| **two shells sharing one face slot is refused, and the test names a narrower case** | `validate.rs:427-433` counts face uses with `for shell in shells.values()`, so the count is **global across shells** and `Invalid::UsedMoreThanOnce { entity: "face" }` catches a slot two shells both list. The only test, `a_shell_that_lists_the_same_face_twice_is_caught` (`validate.rs:2418`), exercises **one** shell listing a face twice. The invariant is wider than the test that names it, and the wider half is the half a consumer met — a face arriving twice with two placements, which downstream is a wrong picture drawn confidently. The guarantee also holds only for a model that has been through `validate()`, which the deserialize door does but a hand-built `Model` need not |
| **why the pointed cone's old ladder was ragged rather than one floor or one band** | each individual refusal is settled; the *pattern* was never worked out, and the file now says so rather than carrying the `√d`-against-`d` sentence that did not fit the data |
| **the frustum tightening's own demonstration** | §52.7's new family covers it now, but only at the patch level. No solid-level test exercises a frustum band the way `a_cone_patch_that_reaches_its_apex.rs` covers a pointed cone |
| **`three_surfaces.rs` wants two flat surfaces** | unchanged. Still the wall for `AFrustumAgainstABar`, 96 of 96 |
| **one pipe, two names, from an origin slid along its own line** | unchanged; §51.7 |
| **`fuse(A, B)` and `fuse(B, A)` refuse with different provenance** | unchanged; §51.7 |
| **`cuts.rs` and most of `three_surfaces.rs`** | unchanged, still unaudited; §51.7's warning about the grep stands |
| **the fuzz decoder draws 3 of 5 kinds** | unchanged; §51.3 |
| **the rest of §51.7** | unchanged |

### 52.12 What was learned about running the work

- **Both gates found the one regression and no lane did.** `a_solid_with_a_cone_face.rs`
  pinned a refusal that the cone fix legitimately removed, and the lane that made
  the fix never ran the workspace suite. Two gates, split by lane rather than by
  numbers-versus-claims, each found it independently on their first full run.
  §51.8's *two gates beat one* holds for a second unit, with a different split.
- **A spike with an empty write set costs nothing to run concurrently.** Row 4's
  file was unknown, which normally serialises it behind whichever lane might own
  it. Made read-only, it ran beside all three lanes and returned the answer that
  the row needed no lane at all.
- **Correcting a brief's own premise before dispatch worked twice.** The row-4
  spike was told its row's expected fix looked already-present, and told to check
  that rather than believe it. It checked, and came back with the prediction that
  settled it. The same brief shape gave the row-1 lane §51.7's false clause as
  *something to establish before fixing*, and it established it was wrong.
- **A lane that would not substitute one confident sentence for another.** Told
  that its `√d`-against-`d` mechanism was contradicted, the lane declined to write
  the gate's replacement mechanism either, because it could not establish that one
  without instrumenting dead code. It wrote that the pattern was never worked out.
  That is the right answer and it should be easier to give than it is.
- **Handbacks stayed cheap because write sets were named first.** Six lanes, seven
  merges, zero conflicts. Four handbacks, and three became another lane's whole
  deliverable.

### 52.13 The prediction for the next unit

§51.6's prediction fired and its countermeasure was shown to have a hole — a
number can be produced by exactly the test that was named, and the test can be
blind. So the next one is about tests rather than sentences:

> **The next unit will find a test that passes for a reason other than the one it
> was written for.** Score it by taking every test whose failure would be the sole
> evidence for a published bound, and mutating the thing it claims to watch — not
> the code near it. A test that stays green under that mutation has been found.

The countermeasure is cheap and it is already written down in two places this
unit: a test earns its keep only if somebody has seen it fail for its own reason.
`a_waved_through_walk_leaves_a_record.rs`'s anti-vacuity halves were mutation-checked
and survived; the cone bound's §15.6 search was not, for years, and did not.

**The three specimens have a sharper shape in common than "a weak test".** Not one
of them is a weak assertion. Each is a *strong* assertion about something smaller
than the doc above it promises:

- a `HashSet` of faces, standing for an order (§52.11);
- a triangle family sitting exactly where two formulas agree, standing for a
  bound (§52.7);
- `a_shell_that_lists_the_same_face_twice_is_caught`, standing for a counter that
  runs over every shell (§52.11).

So the thing to look for is not a test that asserts too little. It is a **test
whose name describes a narrower case than the invariant it stands for**, which is
findable by reading names against docs and needs no mutation to spot.

### 52.14 A wrong unreachability claim is worse than none

The rule in CLAUDE.md already says a claim that something cannot be reached needs
a demonstration and not a paragraph. This unit adds the part about **which way the
error points**, and it earns its own section because the two directions have
opposite costs.

`docs/decisions.md` §16.10's rule is symmetric in the claim and not in the
consequence. A wrong claim that something *can* happen leaves dead code and wastes
a reader's time. **A wrong claim that something cannot happen invites the deletion
of live code, and hands the reader the argument for deleting it.** The comment is
not merely wrong; it is an accomplice.

This unit's own specimen is the spike's negative claim in §52.6 — *only a reach
below `r` could cross the threshold* — which was argued rather than demonstrated,
was backwards, and would have justified not writing the sweep that later measured
159 110 boundary sides. The claim it replaced is a demonstration, and the
difference is not rigour but direction: the second one can be run.

> **An unreachability claim cites where the refusal actually happens** — the
> file and the line that does the refusing — and not where the writer assumes it
> does. Same discipline as citing a test rather than restating it, and for the
> same reason: a cited refusal can be opened and checked, a remembered one cannot.

A consumer hit the same shape twice in one week from opposite directions, and
reported it; that it was found independently outside this repo is why it is
recorded as a shape rather than as one more row.

**A partial justification is the dangerous case, not an absent one.** Applied as
a sweep at the consumer's end the same afternoon, the rule caught a false claim
on its first pass, and the way it had survived is the part worth keeping: the
claim covered **two of four** cases and gave a good reason for those two, so it
read as *more* thorough than a claim covering none — and therefore deterred the
checking that an obviously bare assertion would have invited. A missing reason
advertises itself. A reason that is sound about half its subject does not, and it
is the half nobody looks at that is load-bearing.

So when a negative claim covers several cases, the number it covers is part of
the claim: *none of these four is reachable* wants four citations or an argument
that visibly ranges over all four, and anything less is written as covering what
it actually covers.

**And the way an unchecked fact gets in is that nobody thinks it is a claim.**
The restated measurement in `docs/consumers.md` that §52.11's handedness row
sends a lane to replace did not arrive as a demand or a specification. It arrived
as a courtesy — one side telling the other something it had noticed, offered
rather than filed — and an offered observation does not feel like it needs a
citation to the person offering it, nor like it needs checking to the person
merging it. It becomes a claim the moment it lands in the other side's file, and
by then both of the people who could have doubted it have already decided it was
not that kind of sentence.

> **A fact about rcad written down in rcad's tree is a claim, whatever the tone
> it arrived in.** The test is where it ends up, not how it was offered.

