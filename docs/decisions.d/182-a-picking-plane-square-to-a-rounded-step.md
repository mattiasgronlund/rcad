## 182. A picking plane square to a rounded step misses the lines it should separate, fixed 2026-09-20

*§181 left one question and said it had to be answered before anything was
written: whether making both orders **answer** is less safe than making both
**refuse**. It is measured here, and the answer is sharper than either branch
§181.6 offered. The order that answers is **wrong** — its picking half-space
holds both lines, and its reverse holds neither. The refusal was right all
along. Fixed by reading the picker at both stored axes instead of against a
direction that was rounded on its way in.*

### 182.1 What §181.6 asked, and why the answer is neither of its two branches

§181.6 framed the choice as a safety trade: both orders answering would hand
every consumer a plane too coarse to resolve the slab, both refusing is smaller
and safe under "crash > corrupt". It said which is right "turns on whether any
consumer of a `Picker` at these coordinates can tell the two runs apart".

Mapping the consumers first: a `Picker` built by `two_lines_told_apart` flows
unmodified into exactly one predicate — `which_side_of_a_flat_surface`'s
`normal · p = offset` sign test (`plane.rs:636`) — reached from `walk.rs`'s
`every_half_space_holds` and the fuse's `nothing_rules_the_point_out`
(`fuse/arrange.rs:3255`). Both treat `Undecidable` as *not ruled out*.

But the consumers turned out not to be where the question is settled, because
the plane does not merely fail to resolve the two lines. **It is not between
them.**

### 182.2 The measurement, and four ways it checks out

The two runs are two circles in `(y, z)`: the fat pipe's radius `R1` about
`(0, FAR)`, the thin pipe's `R2 = 1.984375` about `(R1, Z)`. Locating their two
intersection points to 400 digits and reading the **stored** picking plane
there exactly — no `f64` triple anywhere in the loop, so §181.5's correction
does not apply — gives:

| where the plane is read | `normal · p − offset` |
|---|---|
| line `+` | `+8.198839055994672e158` |
| line `−` | `+8.198638727570354e158` |

**Both positive.** `InFrontOf(H)` holds both lines and `InFrontOf(H.reversed())`
holds neither, so the two runs the answering order hands back are "both lines"
and "no lines". That is a wrong answer and not a coarse one.

Four independent checks say the arithmetic is right:

- the two readings differ by `2.003e154`, and divided by `‖normal‖ =
  5.047645e153` that is `3.96875` — exactly `2·R2`, the true gap between the
  runs;
- their mean is `8.198738892e158`, and §181.4 measured the kernel's own
  thin-first residue as `-8.198738891782513e158` — the same sixteen digits, from
  a completely different route, the sign being the negated normal's;
- read at the **fat** pipe's axis the plane gives exactly `0`, which is why
  fat-first certifies; read at the **thin** pipe's axis it gives
  `8.198738891782513e158`;
- the mid-line reading equals `(1−t)·r_fat + t·r_thin` for `t = across/apart`,
  which is the identity the fix below is built on.

In distance the plane misses the axis-to-axis line by `1.62e5`, while the two
runs it has to separate stand `3.97` apart.

### 182.3 Where it is lost, and the claim in the code that is false

`two_lines_told_apart` computed `middle = off_the_picker + leaning · how_far`,
with `leaning = normal · sideways`. For two side-by-side pipes `sideways` is
`towards`, the step across to the other axis, and the picking normal is built
**exactly square to it** — `HalfSpace::through_a_point_square_to(anchor, along,
along.cross(towards))`. So `leaning` is exactly `[-0.0, 0.0]` *by construction*,
`middle` never reads `how_far` at all, and the correction that should carry the
reading from this pipe's axis to the middle of the two lines is identically
zero.

That would be harmless if `towards` were the true step. It is not:
`the_step_across_to_the_other_axis` computes the step exactly and then divides
by `w·w` in `f64`, one rounding per component, and its own doc comment says so.
The normal is square to the **rounded** step; the true step leans out of that
plane by `8.2e158` in reading units, which is `1.62e5` in distance against runs
`3.97` apart.

So this, in `between_the_two_lines`, was **false** and is now deleted:

> *A worse point costs refusals and never a wrong answer, which is why this is
> plain `f64` with no enclosure anywhere in it.*

It costs a wrong answer whenever the check that is supposed to catch a worse
point is blind to the direction the point moved along — which is exactly the
case whenever `leaning` is zero by construction.

**The plane-and-pipe caller never had this.** There `sideways` is the cutting
plane's own **stored** normal, the true step really does run along it, and
`leaning` is a real number rather than a rounding. §181.5 guessed that a pair
with a leaning direction "would feel" the cancellations it reverted; the
opposite is the case — a leaning direction is the sound one.

### 182.4 The fix: read the picker at both stored axes

The middle of the two lines sits `across/apart` of the way from one axis to the
other, so the reading there is that same share of the way between the readings
at the two axes — and both axes are **stored** points, so `Exact` sums both
readings with nothing cancelling. `along`'s own component of the step
contributes nothing to either, the picking normal being exactly square to it.

`TwoLinesEitherSideOfThePicker` now carries `to_the_middle` — how much the
reading changes between `from` and the middle — instead of a direction and a
length, because **what is exact differs between the two callers** and only each
caller knows which. The plane-and-pipe site keeps its `leaning · how_far`; the
pipe-and-pipe site takes the difference of two exact readings. `how_far_in_front`
is the one shared reading.

`#[non_exhaustive]` is not in play — the struct is private to `intersect.rs`.

### 182.5 Both orders refuse, and that is the right answer rather than the safe one

§181.2's seed now refuses both ways round, and
`two_pipes_far_from_the_origin_name_their_crossing_the_same_both_ways_round.rs`
is the regression test. It asserts the two orders **agree** rather than
asserting an answer, because order-independence is the invariant the fuzzer
caught and it is restored by either direction; which direction it went is
recorded here and not pinned there.

This is not the "smaller change under crash > corrupt" §181.6 expected to be
choosing. The answer that used to come back was unsound, so removing it is a
correctness fix and not a conservative one. **§140.5's question is settled for
this family**, in the branch §181.5 raised and then set aside for want of
evidence that its own route could supply: the answering order was certifying a
separation it had not got.

`just gate` passes on the merged tree, 2116 tests, and no published refusal rate
needed changing.

**It refuses the pairs it cannot read and not the family**, which is the
difference between this and widening a bound until the fuzzer goes quiet. All
seven `crash-*` artifacts under `where_two_surfaces_cross` now come back without
a deadly signal, and they do not come back the same way: §181's
`crash-0f06237e`, `crash-477e4468` and `crash-b76d3903` refuse both ways round,
while its fourth — `crash-87e70e7f`, the one whose *thin* axis carries the large
displacement rather than the fat one — **answers** both ways round in two curves,
and the three older artifacts answer exactly as they did. A check that had gone
blanket-conservative would have taken all seven.

### 182.6 What this does not close

**§140's pair still answers, and this does not make the far-from-the-origin one
answer.** The gap §181.6 named stands exactly where it was: a plane through both
axes is not representable as one `f64` offset at these coordinates, because the
two axes' exact readings differ by `8.2e158` while one ulp of the offset is
`3.5e159`. No single offset satisfies both. That is still the
anchor-the-half-space-at-a-point fix, still not excluded, still not built, and
§39.4's row for it stays a **gap** and not a limit.

What changed is that the kernel now **refuses** that pair instead of answering
it wrongly, which is where §16.10 says an unreached answer belongs.

Commands this section was written from: `just decision 181`, `181.2`–`181.7`,
`140.7`, `29.8`; `just src two_lines_told_apart`, `HalfSpace`, `Picker`,
`through_a_point_square_to`, `SurfaceMeet`, `which_side_of_a_flat_surface`; a
400-digit `Decimal` location of the two lines and a `Fraction` reading of the
stored plane at both axes, both in the scratchpad; and `just gate quiet`.
