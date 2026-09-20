## 118. There are two cylinder/cylinder classifiers and the live one has no self-touching check, measured 2026-09-17

§117.2 recorded the six undecided files as a pair no test here answers, and said
the fix family was "a marcher that splits the curve at the two singular points".
Measured on `90006e7b`, that is the wrong family, and the reason is a split the
record did not mention: **this kernel classifies a cylinder/cylinder crossing in
two different places, the two disagree about this family, and the one STEP
import actually runs is the one without the check.**

Nothing below changes a count in §117. The 26, the 20 and the 6 all reproduce on
`90006e7b`. What changes is what the 6 *are*.

### 118.1 The six are exactly degenerate, not nearly so

`kicad/Relay_SPST_SANYOU_SRD_Series_Form_A.step`, the pair the census names for
that file, is `#54` and `#86`: both `CYLINDRICAL_SURFACE`, both radius `0.1`
written as the same literal, so `ρ₂ − ρ₁` is bit-zero and no rounding is
involved. The axes are `(1,0,0)` through `(1.75,5.85,-3.3)` and `(0,-1,0)`
through `(1.85,6.25,-3.3)`. Those two lines meet at `(1.85,5.85,-3.3)`: the
first is `(t, 5.85, -3.3)`, the second is `(1.85, 6.25-s, -3.3)`, and they agree
at `t = 1.85`, `s = 0.4`. The common normal is `(0,0,-1)`, and the two `z`
coordinates are the same literal `-3.3`, so their difference is exactly `0.0`
whatever the decimals round to — the axis gap `δ` is bit-zero as well.

So this family is exactly degenerate. It is not a near miss that a tighter
filter would resolve, and it is not a case where a scaling would reach a sign.
Both quantities the question turns on are exactly zero in `f64`.

That is one file worked out by hand. **All six say the same**, once the census
was made to print the numbers it had been discarding (§118.5): every one is two
equal radii to the last bit and an axis gap of exactly `0.000000000000`, all at
`90.000000` degrees. Four are radius `0.1` and two — the Omron `G6K-2P` pair —
are radius `0.2`, which is worth knowing because it rules out the one remaining
way this could have been an accident of a single value.

### 118.2 The path import takes never asks the question `intersect.rs` answers

`crates/rcad-geom/src/intersect.rs`'s `where_two_leaning_pipes_cross` (`:1083`)
asks exactly this. It compares `ρ₂ − ρ₁ − δ` through the certified cascade —
`ask(&reach(...), Measures::Length, &at, policy, log)`, exact arithmetic
underneath — and on `Decision::Zero` it refuses by name with
`Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo` (`:1117`). Every `Ok`
arm there requires a strict `Pos` or `Neg`. On the data in §118.1 it would
return that error.

It is never called. Its only caller is `FaceView::crossed_with`
(`crates/rcad-topo/src/cross.rs:82`), in a module carrying
`#![allow(dead_code)]` and, at `cross.rs:18-25`, "Nothing in this crate calls it
yet ... Its consumer is named and it is not in this milestone." That is §13
working as intended, and it is also why the check has never run on a real file.

**Corrected 2026-09-17, on §118.11's lane's report.** The two sentences above
are wrong about *which* function has the dead caller, and the sentence "it is
never called" is false as written. `where_two_leaning_pipes_cross` **is** called,
from `where_two_pipes_cross` (`intersect.rs:868`), in the live `Decision::Pos`
arm of the two-pipe dispatch. What has `FaceView::crossed_with` for a caller is
the public entry two frames above it, `pub fn intersect` (`intersect.rs:151`).

**The conclusion the section rests on is unaffected**, and it is the reachability
of the entry, not of the leaf: no path from `Import::read` reaches `intersect`,
so the named refusal never runs on an imported file. Everything below this
paragraph stands. But read the claim as "import cannot reach this classifier",
which is what was measured, and never as "this function is dead" — a lane read
the loose version, went to check it against the file, and found it false, which
is exactly the cost §16.10 is about. A reachability claim names the frame it is
about.

What import runs instead is `rcad_geom::walk`:

```
Import::read → Model::validate → winding_of
  → the_ring_a_wire_traces (rcad-topo/src/validate.rs:2132)
  → rcad_geom::walk → follow_the_quartic
  → march (rcad-geom/src/walk.rs:768)
```

`walk`'s own `[Surface::Cylinder, Surface::Cylinder]` arm (`walk.rs:648-674`)
handles parallel axes, then falls unconditionally to
`a_quartic_picked_up_at(where_one_pipe_reaches_across_another(&thin, &fat))`.
It never reads `run`, and it has no equal-radius case at all. So the pair is
seeded and marched, and `march` refuses at the tangency with "this crossing runs
where its two surfaces are too nearly parallel to follow it" (`error.rs:517`).

`march(` has exactly one non-test call site in the whole workspace, so this is
the only path any imported file can take.

**The message is true about the tangent direction and misleading about the
configuration.** The surfaces are not nearly parallel along the run; they are
exactly tangent at exactly two points, and the curve runs through itself there.
The kernel already has a name for that shape and this path cannot reach it.

### 118.3 The answer is two ellipses, and the file already carries it

Two cylinders of the same radius `r` whose axes cross, directions `e` and `u`.
Subtract the implicit forms and the quadratic part cancels:

```
|p|² − (p·e)² = r²   and   |p|² − (p·u)² = r²
⟹ (p·u)² − (p·e)² = 0
⟹ (p·(u−e)) · (p·(u+e)) = 0
```

That is two planes through the point where the axes cross — the bisector planes
— so the crossing is two ellipses, for **any** crossing angle and not only the
90° these six have. Each has semi-minor `r`; the semi-majors are `r/|sin(θ/2)|`
and `r/|cos(θ/2)|`, equal at 90°.

The file agrees, which is the part that settles it. The refused edge is
`#21 = EDGE_CURVE('',#22,#24,#26,.T.)` and `#26 = ELLIPSE('',#27,
0.141421356237, 0.1)` — semi-minor `0.1`, semi-major `0.1·√2`, exactly what the
formula gives. `Walk::run_along` (`rcad-step/src/brep.rs:738-741`) already
accepts `"ELLIPSE"` and stores it as an `EdgeRun`. The importer parses the right
answer, stores it, and then the validator throws it away to march a quartic.

The shape of the fix is already in the file next door. One arm below the broken
one, `[Surface::Cone(cone), Surface::Cylinder(pipe)]` (`walk.rs:677-683`) is
also a degenerate pair whose crossing is two conics, and it reads the stored run
to say which: `where_a_coaxial_cone_and_pipe_cut(&cone, &pipe, run, policy,
log).map(WhatARunLiesOn::Conic)`, under the comment "this arm reads the run:
which of the two circles it is, is the picker's to say".

### 118.4 So it is a gap, and §13 cannot defer it

§16.9's test: a refusal is a **limit** when the answer does not exist, and a
**gap** when it does and the arithmetic could not reach it. The answer here
exists, is exactly representable, and needs no marching — so it is a gap, and
§13's no-named-consumer rule may not defer it (§16.10 says so in as many words).

It also needs no reversal of §24.10. Nothing in this family is undecidable: the
degeneracy is proven exactly, and the two ellipses follow in closed form.
`march.rs:931`'s `marching_from_where_a_crossing_runs_through_itself_has_no_direction_to_follow`
stays right about what a *marcher* can do at a singular point. The answer is to
not be marching.

### 118.5 Why nobody could check it: the census throws its own evidence away

`crates/rcad-step/examples/near_parallel_which_pair.rs` computed a `detail`
string with the axis gap and the angle, and then the undecided branch `return`ed
`PairEvidence::cannot_tell(...)` before that string was ever used. So all six
printed their prose with no numbers attached, and "axes that pass through each
other" in §117.2 could not be checked by a reader. This unit fixed it: the
undecided branch now prints the two radii, the angle and the gap, which is what
§118.1's second paragraph is read off.

Its `close` is `RADIUS_RELATIVE_TOL = 1e-9` — nine orders looser than the
exactness `intersect.rs` requires. So §117.2's claim was made at `1e-9`, and it
happens also to be true exactly, and nothing recorded which of the two had been
established. That is the §16.10 lesson again in a new place: do not assert what
the evidence you kept can carry, and keep the evidence.

### 118.6 What §117.2 should now be read as saying

Its counts stand and its verdict stands: the six are not transverse, and folding
them into transverse would still be wrong. What does not stand is the last
paragraph. "A marcher that splits the curve at the two singular points" is one
fix family, and it is not the cheap one or the right one; the closed form is.
"Nobody is building that yet" was true of the marcher and is not a reason to
leave the gap open.

Read §117.2 with this section beside it.

### 118.7 The duplication itself stays, with a test on it

Converging the two classifiers is not this unit's work: `cross.rs`'s consumer is
named and belongs to a later milestone, so deleting the unreachable one is wrong
and wiring import through it is bigger than a lane. What this unit owes instead
is a test that goes red if the two ever disagree about this family again —
generated pairs rather than a named fixture, with a floor that fails on an empty
set, per §16.10's test rule. Chosen by Mattias on 2026-09-17 as Q3 (a) with
(c)'s test.

### 118.8 The closed form emptied the corpus's undecided set, and a floor caught it

Answering this family in closed form takes it off the quartic route, and that
route was where every remaining undecided decision in either corpus arose. The
count goes **31 to 0**.

The 31 were all `side of a half-space`, all on a run that was already refused,
and four files carried them: `Relay_1-Form-A/B/C_Schrack-RYII` and
`Relay_SPDT_Finder_34.51_Vertical`. On the merged tree `just gate` prints, for
`certified` and for all three tolerant budgets, over both corpora, `worst wobble
0e0` and an empty record map. So **a tolerant policy now buys nothing measurable
on this corpus** — not a file, which was already true, and no longer an account
of a refusal either.

That is about one population and not about the word "undecided" in general: the
`side of a half-space` decisions reached through `walk` from `Model::validate`.
It says nothing about the census's `Verdict::CannotTell`, which is a different
measurement on a different route, and §118.9 has two of those left.

`whether_a_tolerant_policy_buys_anything_a_certified_one_does_not` went red on
`left_undecided > 0` rather than passing over an empty population. That is
§113.7's rule earning its keep: the floor is what stops a property quietly
asserting about nothing once the last case is rescued, and it is the only reason
this was noticed in the same unit that caused it.

**Where the claim was written is the part worth carrying.** It was in two test
doc comments — `whether_a_tolerant_policy_reaches_the_half_spaces_on_a_run.rs`
and the corpus file above — and in no section of this document; searching every
phrasing of it here returns nothing. So §16.10's audit pass, which reads this
file, could not have found it, and the stale-claim pass found it only because a
test failed. A measured claim that lives only in a test's prose is one neither
pass covers. Prefer the assertion to the sentence.

Where that measurement goes now is open at the time of writing, as Q4. It went
to the hand-built rings, and §118.8's own wording was too broad until the audit
below caught it.

### 118.9 Four of the six, not six, and the other two are 45° — measured on what was pushed

*Written after the push of `f2590235`, by an audit agent that was not any of the
lanes, which is the rule §117.6 paid for.*

The census does **not** shrink. It is still 26 files, and what moved is inside
the bucket:

| | before this unit | `f2590235`, pushed |
|---|---|---|
| files whose only reason is the near-parallel crossing | 26 | **26** |
| tangent — touch without crossing, a limit | 20 | **24** |
| transverse — they really cross, a gap | 0 | 0 |
| undecided — no test here knows the pair | 6 | **2** |

Two files started importing (kicad 110/323 → **112/323**, nist 1/44 unmoved) and
two others reached this wall for the first time, so the total stayed flat — the
same interaction §117.6 was written about, in the same direction, one unit later.

**Four of the six are decided. Two are not**, and they are
`kicad/Relay_SPDT_Finder_32.21-x000.step` and
`kicad/Relay_SPST_Finder_32.21-x300.step`: equal radii `0.3`, axis gap exactly
zero, and axes **45°** apart rather than 90°.

**That is the same family by §118.3's own algebra**, which cancels the quadratic
part for any crossing angle and gives semi-majors `r/|sin(θ/2)|` and
`r/|cos(θ/2)|` — at 45° those are different from each other, where at 90° they
are equal. So either the new arm does not reach these two, or they refuse on a
different edge than the one the census reads. **Which of those it is has not
been established**, and this section does not guess.

**It was neither, and §118.10 has the demonstration.** The arm is angle-general;
the edge is the one the census names; what fails is one step earlier, in
recognising the pair as the family at all. Read the two guesses above as the
guesses they were.

**Read §118.3's "for any crossing angle" as the algebra it is, not as a claim
about the code.** The algebra holds; whether the implementation covers the
unequal-semi-major case is exactly what these two files put in doubt, and it is
the first thing to measure next. §16.10: the demonstration is owed for the
claim as written, and at 45° it has not been given.

### 118.10 Neither of §118.9's two guesses; the family test cannot pass at 45°, measured 2026-09-17

§118.9 offered two: the new arm does not cover unequal semi-majors, or the two
files refuse on a different edge than the census reads. **Both are wrong.** The
demonstration §16.10 asked for is below, and it is a third thing.

**They refuse on the same family, on the edge the census names.** Both files
hold the *identical* pair — the same twelve numbers, which is why they fall
together:

```
#1146/#1016  origin (18.3, -1.19, 10.4)                   direction (0, 1, 0)          radius 0.3
#1318/#1176  origin (17.227867965644, 8.597867965644, 10.4) direction (0.707106781187, -0.707106781187, 0)  radius 0.3
```

And the file carries the closed form, as it did at 90°:
`#1160 = ELLIPSE('',#1161,0.324717660088,0.3)`, and `0.3/cos(22.5°) =
0.324717660088`. §118.3's semi-major formula is confirmed at 45° **by the file**.

**The arm covers unequal semi-majors.** `reach = radius /
the_long_way.cross(mine).length()` is the sine of the angle the bisector leans
from the axis, computed per ellipse, so it is angle-general as written.
`pairs_of_directions` in `two_pipes_of_one_width_on_crossing_axes.rs` has been
walking `(3,4,0)`-shaped pairs at 16° all along. What was missing was a check on
the **sizes** away from a right angle — every size assertion in that file was at
90°, the one angle where `r/|sin(θ/2)|` and `r/|cos(θ/2)|` are equal, so a
closed form that had assumed the equal case would have passed the lot.
`bores_that_lean_rather_than_cross_square_meet_in_two_ellipses_of_different_lengths`
is that check: `(3,4,0)` against `(4,3,0)`, `cos θ = 24/25`, semi-majors `r√50`
and `r√50/7`. It passes, and with the reach forced to `r√2` it fails while the
90° headline test beside it still passes. **That is the measurement that
falsifies §118.9's first guess**, and the fact that it had to be written is
§118.9's real lesson.

#### What actually happens

`whether_two_pipes_are_one_width_on_axes_that_meet` returns **`false`**, so the
pair never reaches the closed form and falls through to the quartic and the
marcher. Of its two halves:

- **the axes meet** — exactly. `w₁ × w₂ = (0, 0, -a)` and both origins are at
  `z = 10.4`, so the triple product is bit-zero.
- **the widths match** — exactly **not**. `Axis::new` scales every direction to
  length one, and at 45° that is `1/√2`, which no `f64` is. The stored direction
  comes back as `0.70710678118654746171500846685376…`, so
  `‖w₂‖² = 1 − 1.7730231858351654e-16` exactly, against `‖w₁‖² = 1` exactly for
  `(0, 1, 0)`. The question `r₂²‖w₁‖² − r₁²‖w₂‖²` is then `+1.5957208672516487e-17`,
  and the exact arm certifies `Pos`.

  **A better normalize cannot close this, and that is worth knowing before
  anyone tries** — but not for the reason first written here. The stored
  component is `0.7071067811865475`, which is **one ulp below**
  `f64::consts::FRAC_1_SQRT_2`; `Axis::new`'s normalize is not correctly rounded.
  That is a real if tiny defect and it is *not* what holds this family back.
  Even the nearest `f64` does not help: `2c² − 1` is `−1.7730231858351654e-16`
  for the value stored and `+1.3671617315323846e-16` for `FRAC_1_SQRT_2`. No
  `f64` doubles its square to one, so rounding the normalize better **moves** the
  gap and does not close it.

  **This paragraph first said the opposite** — that the normalize already lands
  on the nearest `f64` — on a lane's report, taken without re-deriving it, one
  turn after the same session wrote that a lane's word for a number is not
  evidence. `1/math.sqrt(2)` is two roundings and lands a ulp low, which is how
  both the lane and the check agreed and were both wrong. The correction never
  reached `main`; the gate caught it, because the test carrying the same claim
  was merged beside it and the two halves were only ever run together by the
  dispatcher. **That is the argument for gating the merged tree and not the
  lanes.**

**A tolerant policy does not rescue it, at any working size.** Measured at
`tolerant(1.0)`, `(10.0)` and `(100.0)`: the same refusal, unchanged.
`Strictness::TolerantFallback` is a fallback for a sign the filter left
*undecided*, and this sign is not undecided — it is certified non-zero, on an
enclosure that never straddles zero. Tolerance has no entry point here. Anyone
reaching for one to close this should read that sentence first.

#### The root is one step further down, and `Cylinder` already says so

`Cylinder::side_of` takes the sign of `|(p − o) × w|² − r²`. That is degree two
in `w` on the left and degree **zero** on the right — not homogeneous — so the
surface it certifies is the pipe of radius `r/‖w‖`, and `‖w‖` is whatever the
normalize rounded to. The predicate's own doc comment has said this since it was
written: *"the surface this predicate is certified against is the pipe of radius
`radius / length(stored direction)` — displaced from the one the caller named by
something of order `radius * 1e-16`."* Seven sites in `walk.rs` divide by that
length to recover the width, which is correct given the convention and is what
makes the convention visible.

The consequence had not been drawn: **two pipes written with the same radius
literal are the same width in this kernel only when their axis directions round
to the same `‖w‖²`.** That is why the family works at 90° — `(0,1,0)` and
`(0,0,1)` are exactly unit — and why the test file generates its pairs as
permutations of one triple of magnitudes, which is the only other way to get
there. It is written down in that file's module doc, in prose, and that prose
never reached this record; §118.8's lesson, one section later and unheeded.

So `§118.3`'s "for any crossing angle" is sound as algebra **and** as code. What
fails at 45° is membership: the pair cannot be *recognised* as the family,
because as stored it is not the family — it is two pipes whose widths differ by
`1.5e-17`, crossing in a genuine quartic with two necks that narrow to about
that, and the marcher is not wrong to give up on it.

#### Gap or limit, and what is not yet decided

By §16.10 this is a **gap**: the crossing of the surfaces as stored exists, and
the arithmetic cannot reach it. But the fix family is not in `walk.rs`, and
naming it is what §16.10 requires before anyone writes that this cannot be done:

1. **Make the cylinder predicate homogeneous in `w`** — compare `|(p − o) × w|²`
   against `r²‖w‖²`. Both sides are then degree two in `w`, a cylinder means
   what its own doc says it means regardless of how its direction rounded, the
   seven divisions in `walk.rs` collapse to `r`, and these two files import.
   It also fixes the same drift under `Cylinder::transformed`, which §32.3
   deliberately leaves un-normalized. It changes the certified meaning of every
   non-axis-aligned cylinder in the kernel, so it is a decision and not a patch.
2. **Follow the quartic.** The answer differs from the two ellipses by `1e-17`.
3. **Widen the family test.** Banned by its own doc and by the tolerance rule: a
   pair that is merely close crosses in a quartic, and this is the test that
   keeps those out.

**Option 1 is the only one that reaches the answer, and it is not taken here.**
The question it raises is whether this kernel's surfaces are the ones the caller
named or the ones the normalize rounded to, and that is Mattias's to settle, not
a lane's. Until it is, the two Finder relays stay refused and the census stays
24/0/0/2.

**Settled the same day: Mattias chose option 1, and §118.11 is it.** The two
relays import, and the undecided bucket is empty. Read the paragraph above as
the state when this section was written.

#### Correction: `Plane` is not the follow-on, and this section said it was

The paragraph above used to end *"`Plane::side_of_point` carries the same caveat
for the same reason"*, naming it the obvious next step. **That is wrong, and a
read-only sweep of every direction-length dependence in the kernel is what
caught it.**

`Plane::side_of_point` takes the sign of `n·p − d`, and `d` is never an
independent literal: every constructor co-derives it from the *same* stored
normal — `from_point_and_normal` (`plane.rs:129`) computes
`offset = normal.dot(point − origin)` after normalizing, and `from_coefficients`
and `transformed` carry the pair together. So the expression is degree **one** in
`n` on both sides. Scale `n` by `k` and the whole thing scales by `k`; the zero
set does not move and the sign does not change. It is already homogeneous.

The cylinder's defect is the opposite shape: `radius` is an independent literal,
never re-derived from `w`, compared against a quantity that scales with `‖w‖²`.
Nothing about `Plane` has that shape.

What is true of both — and what the predicate doc comments mean when they say
"the same caveat" — is the weak claim: each is exact about the numbers as
stored, not about the geometry those numbers were rounded from. That is
universal to every unit vector in this kernel and no homogeneity fix touches it.
**Reading the weak claim as the strong one is what produced the wrong sentence**,
and it is the same mistake as §118's own: reasoning about what the kernel "does"
from a doc comment rather than from the arithmetic under it. If normalize's
direction rounding is worth attacking, it is a different question from this one
and wants its own section.

Two dependencies that *are* the cylinder's shape came out of the same sweep, and
both are fixed with it rather than after it: `walk.rs`'s `same_width` question
and `intersect.rs`'s `same_thickness` (`where_two_pipes_side_by_side_cross`).
The second is the **namer**, where `walk.rs` is the walker — the two halves §118
is about — so fixing one without the other would re-create §118 pointing the
other way. `cone.rs` was checked and is homogeneous already, both of its terms
being degree two in `w`.

### 118.11 A pipe is the one the caller named, built 2026-09-17

Option 1 of §118.10, chosen by Mattias on 2026-09-17 off that section's table.
`Cylinder::side_of` now takes the sign of `|(p − o) × w|² − r²‖w‖²`. Both sides
are degree two in `w`, so a cylinder means what its own doc comment always said
it meant — every point exactly `radius` from the axis — whatever length the
direction was written or rounded to.

**kicad import 112/323 → 114/323**, nist 1/44 unmoved, read off the gate's own
run. The two files are the §118.9 pair,
`kicad/Relay_SPDT_Finder_32.21-x000.step` and
`kicad/Relay_SPST_Finder_32.21-x300.step`, and no pinned file stopped importing.
The census's undecided bucket, 2 at §118.9, is now empty: **26 → 24 files at the
wall, 24 tangent, 0 transverse, 0 undecided.** `just gate`: 1952 tests.

#### The three arms, and the bound that did not move

`exactly` builds `‖w‖²` as three `two_product` squares summed, then scales it by
`radius²` and by that product's roundoff in turn — the shape `cone.rs` already
uses. Buffers were re-derived rather than nudged: `LONGEST_DIRECTION_SQUARED = 6`,
`LONGEST_RADIUS_TERM = 24`, and `LONGEST_CYLINDER_TOTAL` loses its old `+ 2`,
which was exactly the old bare `r²`.

`SIDE_OF_CYLINDER_ERROR_BOUND` **stays at `(10 + 80ε)ε`, and that is a
re-derivation and not an omission**: the radius term's rounding chain reaches
`(1+ε)⁶`, and six is under the cross-product side's ten, so ten still governs.
§15.6 wants the arithmetic written down where a published bound survives a
change, and it is, in the constant's own doc.

#### Two sites the brief's own grep missed

The brief named seven `direction().length()` hits. **Only five were width
recoveries**; two are `direction * length().recip()`, unit-direction
normalizations that stay. And the grep missed two sites that alias the direction
first — `where_a_plane_cuts_a_pipe`, whose `radius_down_the_slope` was
`r‖n‖/|n·w|`, the *old* certified radius times the correct lean, and which gains
a `* along.length()` because the true stretch is `‖n‖‖w‖/|n·w|`; and a test
helper. **A write-set table built from a grep is a table with holes in it** where
an expression binds the thing it divides by to a local first. §118.10's own count
of seven is wrong for the same reason and is left standing above as the
measurement it was.

#### What is left in `intersect.rs`, and why it was not done

`intersect.rs` is written **throughout** in the old convention and only the one
`same_thickness` question moved. The rest does not come apart piecemeal:
`certified_radii_enclosing` (`:1294`) returns `rᵢ/√(wᵢ·wᵢ)` and its `enclosing`
arm pairs with `how_two_radii_and_a_gap_compare_exactly`'s `exactly` arm inside
one `Question`. **Changing one without the other lets level 3 certify a sign the
enclosure does not bracket**, which is worse than the disagreement it would fix.
Three live callers at `:1085`, `:1181`, `:1262`, and the docs carrying the
convention at `:12`, `:880`, `:955`, `:1055`, `:2455`, `:2498`, `:2540`.

**There is a prize in that lane and not only a tidy-up.** With `ρᵢ = rᵢ` the
leaning-pipes comparison `ρ₁ ± ρ₂` against `δ = |T|/√K` clears to `(r₁ ± r₂)²K`
against `T²`, with **no `W` anywhere** — and that is the expansion §29 records as
five hundred binary orders wide and unsquarable where it stands. It would stop
existing. `the_question_two_leaning_pipes_ask_is_too_wide_to_square_where_it
_stands` is a self-contained test of `Exact` and survives either way; §29's
*geometric* claim would need revisiting, and §16.10 says a claim that something
cannot be done is owed a demonstration when the ground under it moves.

#### The test that fails without it

`a_pipe_about_a_forty_five_degree_axis_is_exactly_its_own_radius_wide`: a unit
pipe about `(1,1,0)`, and the point `(0,0,1)` exactly one radius out. Its own
`assert_ne!(w₀² + w₁², 1.0)` is the demonstration that the old form's exact value
`2a² − 1` is non-zero, so the old predicate certified `Neg` where the truthful
answer is `Zero`.

Beside it, `a_pipe_written_at_another_axis_scale_is_the_same_pipe.rs` puts every
probe at `(0, 0, r)` on a pipe through the origin lying in the `xy` plane, so
`p − o` is exact and the predicate collapses to `r²(‖w‖² − 1)` — the sign of the
stored direction's own rounding and nothing else. It was written red, against
`235226b3`, by a lane with no write access to `src/`, and the failing sign went
**both ways** across its cases, which is what separates the defect from a probe
point built in the wrong place.

One trap it recorded: `scaled_to_length_one_from_anywhere` divides by the largest
component first, so `(1,2,0)` and `(3,6,0)` store **identically**. A test
asserting that two spellings of one direction agree would pass vacuously. Every
assertion in that file is against `Decision::Zero` — the geometry — and never
against the other spelling.

