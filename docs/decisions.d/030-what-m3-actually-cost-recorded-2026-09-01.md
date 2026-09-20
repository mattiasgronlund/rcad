## 30. What M3 actually cost, recorded 2026-09-01

*Written by M3's closing gate. §12 did this for M0, §16 for M1, §17.8 for the
fuse and §26 for M2. §24.12's shape was "gate, two stages, gate"; this is the
second gate.*

M3 is complete. `rcad-geom` names where any two of a plane, a ball and a pipe
cross, refuses by name where it cannot, walks certified points along the answer,
and marches the two rows no conic stands in for. The mesher no longer walks its
own edges. A solid whose edges were **computed** meshes and passes §23.4's
oracle, and one of them is a shape M2 could not declare at all. The suite went
from 860 tests at M2's close to 985.

`rcad-predicates` was touched once, at the gate, for `Measures::ManyLengths` and
`Cause::PointAlongACrossing`. §24.4 item 4's freeze held through both stages and
through this gate.

### 30.1 The fourteen criteria, and the test that discharges each

§17's table of "criterion → the test that discharges it" is the precedent, and
it is the most useful thing these sections contain. Every row names a test that
fails if the claim stops holding.

| | Criterion | Discharged by |
|---|---|---|
| 1 | the four planar pairs name their crossing exactly | `two_planes_leaning_against_each_other_cross_in_one_endless_line`, `a_plane_through_a_ball_cuts_one_circle`, `a_plane_that_leans_across_a_pipe_cuts_one_closed_curve`, `two_balls_that_overlap_cross_in_one_circle` |
| 2 | every named run passes `validate`'s check | `every_point_of_a_picked_run_is_inside_every_half_space_that_picked_it` — the whole run and not only its two ends — with `an_edge_named_by_a_crossing_it_is_not_on_is_caught` and `an_edge_outside_a_curved_half_space_that_picked_it_is_caught` for the check itself |
| 3 | every evaluated point certified ON both surfaces | `every_point_along_a_crossing_is_on_both_surfaces.rs`, six families, about 130 000 points |
| 4 | `TouchAtOnePoint` and `SameSurface` are answers; roadmap §7.4 is a table | `the_degeneracies_a_crossing_has_to_answer.rs`, fourteen rows, one test each |
| 5 | lattice control: **0%** | `a_certified_naming_of_a_lattice_crossing_never_refuses` and `a_certified_naming_of_a_quartic_lattice_crossing_never_refuses`, both asserted at zero |
| 6 | rotated rate measured and under a recorded threshold | `how_often_naming_a_crossing_refuses.rs`, five families, thresholds in §28.1 and §29.5 |
| 7 | marcher against closed form, as a ratio | `a_marched_circle_stays_inside_the_bound_the_marcher_published_for_it` and `..._written_anywhere_in_f64_stays_inside_the_same_share_of_its_bound` |
| 8 | a computed-edge solid meshes and passes §23.4's oracle | `a_bar_whose_cap_circles_were_computed_meshes_exactly_as_the_declared_one_does` and `a_bar_cut_off_at_an_angle_meshes_into_a_closed_surface`, both through `the_mesh_closes_up`; `a_bar_cut_off_at_an_angle_is_built_and_meshed_from_the_front_door` out at the facade |
| 9 | transform invariance | `turning_a_pair_of_surfaces_names_the_same_kind_of_crossing` **and** `turning_a_quartic_pair_of_surfaces_names_the_same_kind_of_crossing` — the second added at this gate, see §30.4 |
| 10 | a `cargo-fuzz` target over `(Surface, Surface)`, corpus checked in | `fuzz/fuzz_targets/where_two_surfaces_cross.rs`; on every commit, `every_seed_in_the_checked_in_corpus_names_its_crossing_honestly` and `a_soak_over_pseudo_random_bytes_names_every_crossing_honestly` |
| 11 | cost per marched point, divide broken out | `what_a_marched_point_costs.rs`. Human-read, no target (§24.16) |
| 12 | the claim audit passes; every refusal classified | §30.4 and §30.5 |
| 13 | `FORMAT_VERSION` at 2 with a round trip | `a_seam_picked_out_by_a_pipe_comes_back_picked_out_by_the_same_pipe`, `a_run_the_intersector_hands_back_writes_out_and_reads_in_unchanged`, `a_plane_and_a_surface_holding_that_plane_are_different_bytes` |
| 14 | the facade exports the intersector, `SurfaceMeet`, `NamedRun`, `EdgeRun`; `FaceMeet` stays private | `naming_a_crossing_from_the_front_door.rs`: `the_hardest_edge_the_kernel_builds_can_be_said_in_this_vocabulary`, `every_way_two_surfaces_can_meet_is_something_a_caller_can_match_on`, `a_caller_out_here_can_cross_two_surfaces_and_walk_the_answer` |

**All fourteen are met.** Two of them were not when this gate opened: criterion 8
had no builder and no wiring (§29.13 item 1), and criterion 9 was discharged for
four of the six pairs and said it was discharged for all of them.

One thing handed to M3 and **not** done, which is not a criterion and is said
here rather than buried: §26.6 item 4 predicted that `Undecidable` out of `mesh`
would become reachable when a mesh domain first held a constructed corner, and
named M3 as the milestone that could write the test. It cannot. `walk` hands the
mesher 3D points and the mesher reads them back through `Surface::coordinates_of`,
so every corner the triangulator sees is still an ordinary `f64` pair. The first
constructed corner in a mesh domain is a fuse's, and that is M4's.

### 30.2 §24.12's prediction, scored: wrong in both halves

§24.12 named `SideOfSurface` for sphere and cylinder, and the local interval
divide beneath it, as where M3's money would go. It was recorded before the work
on purpose.

- **First half, misaimed before a line was written.** §27.4 read both functions
  at the gate. `SideOfSurface` needed no work at all; what needed care was its
  *callers*, because a discriminant written the textbook way is certified about a
  different ball than the predicate is (§27.1).
- **Second half, wrong by measurement.** There is no interval divide beneath
  `SideOfSurface` — neither sphere nor cylinder divides at any of the three
  levels. §16.8's divide is the spline evaluator's and M3 never calls it. Scored
  against the divide the marcher actually built, which is what §27.4 said to
  score it against: **9 divides at 11.2 ns is 4% of a 2.66 µs marched point**
  (§29.10, re-measured at this gate).

The money went to **settling**: two Newton corrections per trial step, six dot
products and a two-by-two solve each, and a certified enclosure of the deviation
on top.

**The running score.** §26.2 corrected this once already and it stands: §12.1
stated its lesson after the fact and predicted nothing; §17.6 predicted and was
right; §23.7 predicted and was half right, in the sense that acting on the
prediction is what stopped it coming true. §24.12 is the fourth and it is wrong.
**Three predictions made in advance, one right, one half right, one wrong.**

**What the project should conclude, and it is not "stop predicting".** §12.1's
lesson is that the money goes to code that was already written and looked
finished. That held for M0 and M1 and half held for M2, and every one of those
milestones spent most of its time *rewiring* code that existed. M3 wrote
`intersect.rs`, `walk.rs` and `march.rs` from nothing, and the money went to the
new code — to a settling loop that did not exist when the prediction was made.
So §12.1's lesson has a precondition nobody had written down: **it is a claim
about milestones that mostly rewire, and a milestone that mostly writes will not
obey it.** The prediction is what exposed that, which is the second-best thing a
prediction can do.

The other half of the conclusion is cheaper and more immediately useful. Half of
§24.12 was scored *at the gate*, by reading two functions for an hour, before any
of the work it was about. A prediction made in advance is worth what the next
gate can spend checking it against the code, and that checking is far cheaper
than the prediction is accurate. Keep making them; check them at the next gate
rather than at the milestone's end.

### 30.3 What §24 got wrong, and what a settling document is for

Gathered rather than re-derived. §24 was written before anybody had read the
discriminants, and it shows in a particular way.

- **The discriminants are not the textbook ones** (§27.1). They have to carry
  `n·n` and `w·w`, or the intersector and `SideOfSurface` are certified about two
  different solids.
- **There is no near-tangency band, because there is no constructed input**
  (§27.1). §24.10 said the band exists only for constructed inputs; sharper, the
  naming half has none at all, so criterion 5's 0% is structural rather than
  hopeful.
- **"Two ends" is the wrong shape for a run** (§28.3). A pair of surfaces names a
  curve and nothing on it; ends are what topology adds.
- **"Certified ON" is unreachable as written** (§28.4). It has to mean "never
  certified off, and certified off a surface it is plainly not on", which is two
  halves rather than one.
- **§24.5's ball-and-pipe row is entirely nameable** and its pipe-and-pipe row
  less so (§29.11). Every ball-and-pipe crossing is named exactly; the
  equal-radii pipe pair §24.5 lists as a named special case cannot be named at
  all, because it produces one curve that runs through itself.
- **Criterion 7's oracle came from a threaded ball, not from equal pipes**
  (§29.11). §24.5's claim that the oracle exists by construction holds, for a
  different reason than it gives.
- **§24.17's structural claim splits three ways** (§29.6), and only one third of
  it is a limit.
- **Criterion 9's wording outran its generator**, and that one is this gate's.
  §30.4.

**Every one of those except the last is about arithmetic, and every one of them
cost almost nothing.** Four were found in a day at the gate before any code, two
while writing the code, and two by the answer coming out better than promised.
That is §26.3's split arriving a second time: a settling document is wrong about
numbers and right about meanings, and the numbers are cheap to be wrong about
because the first person to need one does the arithmetic. §26.3 ended with *"It
should stop suggesting numbers"*, and §24 suggested numbers anyway. Two
milestones is enough: the convention should be a rule and not an observation.

**The one that cost something is the one that reads like a test.** Criterion 9
says *"the same pair under a general rotation names the same kind of crossing"*.
That is not a claim about a number, so nobody did arithmetic on it; it is a
specification of a property, a property with that name existed, and the property
was green. It survived a gate, two stages and a claim audit's worth of review,
and what found it was enumerating what the generator draws. **The dangerous
sentences in a settling document are the ones that specify a test rather than
claim a number**, because a number gets checked by the first person who needs it
and a test that already exists does not.

### 30.4 What the claim audit found

The audit (`claim-audit`, per §16.10) ran over M3's seven new files in
`rcad-geom` and the mesher wiring in `rcad-topo`. §24.17 said M3 makes more of
these claims than any milestone so far, and it does. Three defects and four
sentences.

**Defects.**

1. **Criterion 9 covered four of the six pairs.** `a_lattice_crossing`'s own doc
   says a ball crossing a pipe and a pipe crossing a pipe are drawn by
   `a_quartic_pair` instead, and nothing was handing that strategy to the
   invariance property. A **sufficiency** claim, broken by enumerating what the
   generator ranges over. The quartic pairs have one knife edge the planar ones
   do not — a crossing that runs through itself is a tangency, so a turn moves it
   off — and it is skipped alongside the others. Measured: **1594 of 2048
   compared, 455 skipped**, against 1485 and 564 for the planar family.
2. **`walk` panicked on a caller's argument.** A run's ends are `Point`s, so an
   end may be a box; the box whose every side runs the whole number line is how a
   pair of surfaces says it names a curve and nothing on it, and **every other
   end has to have a middle**. `NamedRun::new` takes an exact triple with a `NaN`
   in it, and a box with one infinite side, and both make the angle round the
   conic not a number — down to `Interval::new(NaN, NaN)` and a panic. An
   **unreachability** claim by silence: `walk` is public, `NamedRun::new` is
   public, and nothing between them narrows what an end may be, which is §16.10's
   rule about checking the public API rather than today's callers.
   `Error::AnEndOfARunNamesNoPlace`.
3. **A refusal the second stage made unreachable and left behind.**
   `CrossingRunsToAQuartic`'s doc said the evaluator still raised it. It does
   not: `walk` matches the three quartic pairs and hands every one of them to the
   marcher before the closed form is reached, so the arm that produced the
   variant covers exactly the set that never arrives. Removed; the arm is a panic
   naming the invariant.

**Sentences, corrected rather than fixed.**

4. **"Fifty halvings puts any step below the smallest normal number"** is out by
   twenty binary orders: `2^-50` of a step of one is `9e-16` and the smallest
   normal number is `2e-308`. Fifty is the right number for a different reason —
   the sagitta falls with the *square* of the step, so fifty halvings ask a piece
   to bend `1e-30` of what the first one did — and the doc says that now.
5. **"Every accepted step moves the point at least `step` along the curve"** is
   not what the code checks. The trial point is settled onto both surfaces
   afterwards, so what a step covers is whatever the settling left, and all that
   is asked is that it is not nothing. The termination argument leans on the
   count cap, which the same paragraph already said.
6. **"Level 3 decides the rest"** is false for the one question that is not a
   polynomial. Two leaning pipes compare a length against a length in three
   different metrics; clearing the roots costs eight degrees and runs past what
   `f64` can square, so the exact arm answers `None` and the enclosure is the
   whole of the answer (§29.2).
7. **"Nothing builds a quartic yet"**, on `EdgeRun::RightRoundWhereItsFacesCross`,
   stopped being true when the second stage built them. §29.13 item 3 had
   noticed; the doc comment had not.

**What it could not break.** The hardest push went at §29.6's structural claim —
*an expansion whose entries span more than about half of `f64`'s range cannot be
squared at any scale*. Squaring needs every pairwise product exact, so the
smallest entry must be above `2^-484` and the largest below `2^512`, which is a
spread of about 996 binary orders; scaling by a power of two slides both together
and cannot change the spread. The claim is right, "about half of `f64`'s range"
is a fair reading of 996 against 2098, and §29.6's measured 505-order spread is
correctly a **gap** rather than a limit. The second-hardest went at
`how_many_steps`'s bound `reach · (1 − cos(step/2))` for an *ellipse* rather than
a circle: it holds, and it is tight at a half turn and at two thirds of one for a
degenerate ellipse, because the two semi-axes are square to each other and the
second derivative of the parametrization is never longer than the larger of them.

### 30.5 Every refusal M3 can produce, limit or gap

§24.17: *"The claim audit runs before the closing gate, and every refusal is
classified."* Fourteen rows, because four refusals have two causes each and
§17.7 says families are reported separately. **Six are gaps and seven are
limits, and one is filed as both**, because §29.6 says the naming half and the
evaluator classify it differently.

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `Undecidable` | a product leaves `f64`'s range while naming | **gap** | `a_ball_too_small_for_its_own_squares_cannot_be_crossed_and_the_same_one_scaled_up_can`; all 215 refusals in the shipped corpus are named on the lattice by `every_crossing_a_scaling_refuses_is_one_the_same_shapes_on_the_lattice_can_name` |
| `Undecidable` | a direction with a subnormal component underflows its own product | **gap** | `a_direction_with_a_subnormal_component_cannot_be_told_from_square_to_a_plane` |
| `Undecidable` | two leaning pipes' comparison is too wide to square where it stands | **gap** | `the_question_two_leaning_pipes_ask_is_too_wide_to_square_where_it_stands` — the same expansion written `2^300` higher squares |
| `CannotNameWhichPieceOfACrossingThisIs` | a picking normal must be exactly square to an endless direction and `Plane` scales every normal | **gap** | `two_lines_no_f64_half_space_can_tell_apart_are_refused_rather_than_guessed`; the same pair on the lattice is named. §29.8, and M4's to close |
| `CannotNameWhichPieceOfACrossingThisIs` | the half-spaces on a run pick both arcs between its ends, or neither | **limit** | `a_seam_that_picks_neither_half_of_its_meridian_or_both_is_refused` — the two halves of that meridian are genuinely alike |
| `ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo` | the two surfaces are tangent along the curve | **limit** of the naming half taken with the evaluator; **gap** of the naming half alone | `marching_from_where_a_crossing_runs_through_itself_has_no_direction_to_follow`; §29.6 |
| `CannotWorkOutHowBigACrossingIs` | the conic's own size runs past the top of `f64` | **gap** | `a_crossing_too_big_to_put_a_bound_on_is_refused_and_the_same_one_smaller_is_walked` |
| `CannotFollowACrossingWhereItsSurfacesAreTangent` | the two surfaces really are tangent where the marcher stands | **limit** | `marching_along_a_crossing_where_the_two_surfaces_are_tangent_refuses_by_name` |
| `CannotFollowACrossingWhereItsSurfacesAreTangent` | the sine of the angle between them cannot be told from zero, though they do cross | **gap**, and **no generator reaches it** | Named and not demonstrated. §29.7's `Exact::enclosure` took the band family from refusing 0.2% to refusing none, and nothing in the suite has reached this arm since |
| `CannotPlaceAPointThatCloseToACrossing` | the deflection asks for closer than a point can be placed | **limit** | `a_tolerant_caller_gets_the_finest_mesh_the_arithmetic_reaches` — a certified caller is refused and a tolerant one is told what it got |
| `TooManyPointsAlongOneCrossing` | the request needs more points than one run holds | **limit** of the request | `a_crossing_that_takes_more_points_than_it_is_allowed_refuses_rather_than_stopping_short`, `a_deflection_needing_more_points_than_can_be_held_is_refused` |
| `AnEndOfARunNamesNoPlace` | an end that is neither "nowhere in particular" nor a box with a middle | **limit** | `an_end_that_does_not_say_where_it_is_is_refused_rather_than_walked` |
| `ACapDoesNotCutRightRoundABar`, `ACapOfABarFacesIntoIt`, `TheCapsOfABarMeetInsideIt` | two planes that do not cut a bar | **limit** | `a_bar_whose_caps_are_not_caps_is_refused` |
| `AnEdgeWithoutTwoFacesAlongItCannotBeWalked` | three or more faces meet along one edge, so no pair says where it runs | **limit** | `an_edge_with_three_faces_along_it_is_refused_rather_than_asserted_against` |

`DeflectionBelowWhatAMeshCanReach` is M2's and §26.5 item 3 already classified it
a **limit**. M3 moved its floor and §30.6 says by how much.

**The one row worth reading twice is the ninth**, because it is the only refusal
in the milestone that is named, classified and undemonstrated. It is a gap by
construction — two surfaces that cross have a direction to follow, and only the
arithmetic loses it — and the generator built to reach exactly that band
(§29.11's `a_crossing_a_hair_from_the_knife_edge`) does not, because
`Exact::enclosure` made the sine decidable at scales nobody expected. That is a
claim about unreachability with a measurement behind it rather than a paragraph,
which is what §16.10 asks for, but it is not a demonstration and it should not be
written up as one.

### 30.6 What wiring the mesher to `walk` moved

§28.9 item 5 left this to the closing gate and predicted that swapping the mesher
over "changes how many points every curved mesh gets". Measured, before and
after, at a 100 mm working size:

| | before | after |
|---|---|---|
| bar, radius 3, height 5, deflection 0.1 | 58 points, 112 triangles | **58, 112** |
| the same bar at 0.01 | 178, 352 | **178, 352** |
| ball, radius 3, deflection 0.1 | 422 points, 840 triangles | **424, 844** |
| the same ball at 0.01 | 4463, 8922 | **4429, 8854** |

**The bar did not move at all and the ball did**, and the reason is worth having.
The two layers measure the same arc differently: the evaluator sizes a step by
the true sagitta of the curve, `reach · (1 − cos(step/2))`, and `rcad-mesh` judges
what comes out by `chord² / (3 · radius)`, the bound it derives for a flat piece
with its corners on any surface. Those differ by exactly eight thirds, because
`sin²x = (1 − cos x)(1 + cos x) ≤ 2(1 − cos x)` — so a boundary is refined to
three eighths of what the triangulator will allow, and that factor is an
inequality turned round rather than a fudge. With it in place a bar's cap circle
comes out at the same count either way. A ball's seam does not, because it is
half a great circle and the old walk sized it off the ball's radius through the
chord formula while the new one sizes it off the conic's own reach. Both meshes
pass the oracle at both deflections, and the ball's **measured volume moved
towards the exact `4πr³/3` in both directions**: 111.27610 → 111.29357 and
112.89652 → 112.89801, against 113.09734.

Three things came out of the wiring that were not wiring.

1. **A conic carried one number where it needed two.** The size everything was
   worked out from — a plane's offset, a ball's centre, an axis origin — is the
   right scale for a *rounding* and the wrong one for a *bend*. Charged against
   it, the cap circle ten up a bar was walked into more pieces than the one at the
   bottom, and the middle of a meshed bar was no longer half way up it. §19 for
   the evaluator, and it is `how_finely_a_crossing_is_walked_is_about_its_size_
   not_its_place.rs` now.
2. **The floor under a deflection is two chains in two frames.** The evaluator
   follows a crossing in the frame the face is *stored* in and the placement
   rounds against where the solid *sits*, and charging both against the far one
   would have put the floor fifty times higher than the arithmetic asks for.
3. **Which meridian a pole-to-pole seam runs down is no longer read in
   `rcad-topo`.** It is the same question `walk` already asks — which of the two
   arcs between the same two ends is this — and asking it twice is the shape §23.8
   kept finding. `Error::ASeamBetweenTwoPolesNamesNoMeridian` is gone and the
   refusal comes from the evaluator.

So §28.9 item 5's "what is left is wiring" was right about the seam and the
types, and wrong that nothing would be found: a defect in `walk`, a derivation
that had to be done, and a floor that had to be split.

### 30.7 The numbers

Reported family by family and never blended, per §17.7.

**Criterion 5, the control.** 2048 pairs per family, `Policy::certified()`.

| Family | `Undecidable` | Threshold |
|---|---|---|
| lattice, planar pairs | **0.00%** | zero, asserted |
| lattice, quartic pairs | **0.00%** | zero, asserted |

**Criterion 6, the headline.** Same 2048 per family. "No half-space" is
`CannotNameWhichPieceOfACrossingThisIs` and is counted apart because it refuses
for a reason `Undecidable` does not have.

| Family | `Undecidable` | no half-space | runs through itself | thresholds |
|---|---|---|---|---|
| turned, planar | 0.00% | 0.44% | 0.00% | zero / 2% |
| turned, quartic | **4.64%** | **10.93%** | 0.00% | 8% / 15% |
| leaning | 0.00% | 0.24% | 0.00% | zero / 2% |
| the band | 0.00% | 10.79% | 1.07% | 5% |
| written at `2^k` | 10.49% | 3.95% | 0.00% | 25% |

**Criterion 3**, over about 130 000 points: the furthest any point sat off its
own curve, as a share of the bound published for it.

| | |
|---|---|
| lattice / leaning / turned, parametrized | 0.006 / 0.008 / 0.012 |
| lattice / turned quartic, marched | 0.738 / 0.355 |
| the band | 0.410 |

**Criterion 7**, the load-bearing one: **0.577** of the marcher's own published
bound, and **the same figure at every scale** from `2^-200` to `2^200`. The
ceiling is three quarters. The parametrized arm of the same circles is 0.008,
and §29.4 says why the gap between them is the bound being nearly tight rather
than a weakness.

**Criterion 10**: **901 seconds, 23 037 787 executions at about 25 600 a
second**, corpus minimised to **562 inputs — 529 the fuzzer kept and 33 written
by hand**. On every commit the same property runs over the same decoder: the
whole corpus, plus 200 000 pseudo-random byte strings and about 700 000 points,
in five seconds.

**Criterion 11**, human-read and no target (§24.16), re-measured at this gate:

| | |
|---|---|
| one certified marched point | **2.66 µs** (662 µs for a 249-point run) |
| one parametrized point | 34 ns (2.35 µs for a 69-point run) |
| naming the whole crossing, drawing none of it | 1.48 µs |
| one interval divide | 11.2 ns |
| **the interval divide's share of a marched point** | **9 × 11.2 ns = 4%** |

Within a few percent of §29.10 on every row. Marching still costs about eighty
times parametrizing the same curve.

**The suite.**

| | at §29 | now |
|---|---|---|
| tests (`cargo nextest`) | 973 | **985** |
| suite duration | 18.1 s | 18.0 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | 2 |
| fuzz corpus | 562 | 562 |

**`just fuzz` was re-run because this lane touches `rcad-geom`**: 5 105 002
executions in 181 seconds at about 28 200 a second, and **no defect at all**.
Coverage moved by one edge and the working corpus by four inputs, none of which
is worth checking in unminimised, so the shipped corpus stays at 562.

Twelve tests and no time at all, which is what a gate that fixes things rather
than building them looks like. Across the milestone: **860 → 985 tests, 17.0 s →
18.0 s.**

### 30.8 What M3 hands to M4

§24.19 wrote five items before the work. Corrected and extended.

1. **`classify` and `fuse` stay gated on `is_wholly_flat`** (§24.19 item 1,
   unchanged). M4 is where a fuse over curved crossings is the first consumer
   that can remove the gate.
2. **The interval divide is measured and it is not where the money is.** §24.19
   item 2 said M4 would be the first caller with a figure to beat. The figure
   exists and it says 4%; what to optimise, if anything, is **settling** — two
   Newton corrections per trial step, each six dot products and a two-by-two
   solve, with a certified enclosure of the deviation on top.
3. **The general quartic is named and only ever marched**, so a fuse over two
   crossing pipes is where criterion 7's 0.577 is first spent in anger (§24.19
   item 3, unchanged).
4. **A working frame now has a figure asking for it**, and §24.8 deferred it on
   the explicit grounds that it had none. The figure is §29.6's **505-order
   spread**, and the ask is specific: the two-pipe discriminant needs its
   *expansion slid up the range*, not its inputs centred. That deferral has come
   due. `rkyv`, the bump arena, cone and torus and the B-spline fit stay deferred
   with their consumers named.
5. **`Plane` storing an unscaled normal** has a measured cost of not doing it:
   0.44% of turned planar pairs and 10.9% of turned quartic ones. It is a change
   to a type every predicate in the kernel is certified about, which is why it is
   M4's.
6. **A crossing that runs through itself** needs a third `EdgeRun` variant and a
   marcher that can be told to start away from a singular point and stop at it.
   One consumer, and it is M4's fuse (§29.6).
7. **`Measures::Dimensionless` is still one line** (§28.6), and the leaning
   pipes' three comparisons are lengths, which is the right arm and the wrong
   name.
8. **The mesher's floor moved by a factor of about fifty.** A boundary point is
   placed by the evaluator now, which publishes `256 · ε` where the mesher
   charged `8 · ε`, so the finest mesh of a solid at the origin is that much
   coarser than it was. Nothing needs it finer today. The way back is a tighter
   derivation of `POINT_ALONG_A_CROSSING_ROUNDOFF`, which has seven times the
   room its own chain needs (§28.5) and is wide for the trigonometry.
9. **`MOST_POINTS_A_MESH_WILL_HOLD` is shadowed by the per-crossing cap.** A ball
   has one curved edge and `MOST_POINTS_ALONG_ONE_CROSSING` is four times
   smaller, so the whole-mesh cap is now only reachable through the cutting of a
   face. The first solid with five curved edges is a fused one.
10. **A closed edge that is a *hole* in a curved face has no direction rule.**
    The mesher picks a closed edge's direction by the sign of its turn round the
    face, which is what makes a bar's wall close; a hole that does not wind round
    the face gets the evaluator's own direction, and nothing says that is the one
    which reads as a hole. No builder makes such a face. M4's ball with a pipe
    through it is the first.
11. **`Undecidable` out of `mesh` is still unreachable**, and §26.6 item 4 named
    M3 as the milestone that would reach it. §30.1 says why it did not, and the
    first constructed corner in a mesh domain is a fuse's.

### 30.9 Corrections to earlier sections

- **§24.19 item 2** said M4 would be the first caller with a figure the interval
  divide has to beat. The figure arrived in M3 and it says the divide is 4%. See
  §30.8 item 2.
- **§24.18 criterion 9** reads "the same pair", and the property that discharged
  it drew four of the six pairs. Both are drawn now. §30.4.
- **§26.6 item 4** is not closed and M3 could not close it. §30.1.
- **§28.9 item 5**'s "what is left is wiring" was right about the seam and the
  types and wrong that the wiring would find nothing. §30.6.
- **§28.7** lists `CrossingRunsToAQuartic` among the four error variants stage
  one added. The second stage made it unreachable; it is gone. §30.4 item 3.
- **§29.9** says the corpus is "530 the fuzzer kept and 32 written out by hand".
  It is **529 and 33**; the total of 562 is right.
- **§29.10**'s benchmark figures are confirmed within a few percent. The
  parametrized run is 69 points and not 70, because §30.6 item 1 changed which
  number the step rule reads.
- **§29.13 item 1** says criterion 8 is not done and **item 2** says the claim
  audit has not run. Both are done. **Item 3** stands: the curve that runs
  through itself is still outside `EdgeRun`'s vocabulary, and §30.8 item 6 hands
  it on. **Items 4 and 5** are carried into §30.8 items 4 and 5.

---

