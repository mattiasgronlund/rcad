## 183. The picking normal, not the offset: what the far-from-the-origin pipe pair really loses, measured 2026-09-21

*§182.6 left the gap as "no single `f64` offset holds a plane through both axes
out there", and named one fix for it: carry the point the half-space was built
through and read `normal · (x − p)`. That is a change to `HalfSpace`, which
every predicate in the kernel is certified about. **It is not the fix, and the
offset's ulp is not the constraint.** Measured here: a picking plane that
separates the two lines exists at these coordinates with a margin of `1.99e243`,
it is one of the constructor's own fallback normals, and its offset is the
stored number `41581179` with no rounding in it at all. Nothing about
`HalfSpace` has to change. §39.4's row stays a **gap**; what changes is which
arithmetic has to reach.*

*Read §183.4 before §183.3's table. That table's third row places the middle
along the step, which is what the kernel did before §182, and it makes the
step's own rounding look load-bearing when §183.4 shows it is not.*

### 183.1 Why §181.6 named the wrong fix, and it is §16.10's own lesson again

§181.6 ruled out rescaling the normal, and it is right: offset and ulp scale
together and the ratio is invariant. Then it treated that as ruling out the
normal altogether and went looking at the offset's representation.

**A scaling is not the only way to change a normal.** The exactly-square family
is two-dimensional, and its members differ in more than length — they differ in
which of their components are zero. That is the whole of it, and it is §16.10's
sentence word for word one level down: "no ordering of the arithmetic helps" was
true of orderings and silent about scalings; "rescaling the normal is excluded"
is true of scalings and silent about **directions**.

### 183.2 What the probe found: four anchors, one offset

`two_lines_between_two_pipes` was run on §181.2's `crash-0f06237e` with the
anchor swapped four ways — the midpoint it computes today, the fat pipe's stored
axis, the thin pipe's stored axis, and the midpoint taken back from the far end
so that no subtraction cancels. All four produce the **same** half-space:

```
midpoint  = [100663296.0, 5.047645337156084e153, 41418752.0]
back_from = [5.047645337156084e153, 5.047645337156084e153, 41581179.0]

 midpoint/asked: offset=1.870399788587003e175 middle=[8.1987388917825e158, ...] straddles=false
     here/asked: offset=1.870399788587003e175 middle=[8.1987388917825e158, ...] straddles=false
    there/asked: offset=1.870399788587003e175 middle=[8.1987388917825e158, ...] straddles=false
back_from/asked: offset=1.870399788587003e175 middle=[8.1987388917825e158, ...] straddles=false
```

**An anchored half-space would carry exactly this.** Reading `normal · (x − p)`
instead of `normal · x − offset` changes nothing when every candidate `p` a
caller can name rounds into the same plane. §181.6's fourth bullet says such a
reading means "a point near `p` never forms `1.87e175` at all", and that is true
of the *reading*; it is not true of *where the plane sits*, which is what
refuses. The two are different questions and §182.6 ran them together.

That `middle` reproduces §182.2 exactly — `8.198738891782507e158` against the
mean of its two 400-digit line readings, `8.198738891782513e158` — so the probe
is measuring the same plane §182 measured, by a different route.

### 183.3 The plane that does separate them

The same run, with the normal taken from the constructor's own fallbacks instead
of from the direction the caller asked for:

| picker | offset | `middle` | window | verdict |
|---|---|---|---|---|
| `along × towards`, any anchor | `1.870399788587003e175` | `8.198739e158` | `1.001642e154` | short by `8.19e4` |
| `along × ŷ`, thin axis, step as stored | `41581179` exactly | `-1.624270e5` | `1.984375` | short by `8.19e4` |
| `along × ŷ`, thin axis, step taken exactly | `41581179` exactly | `9.984566e-244` | `1.984375` | **separates, by `1.99e243`** |

`along × ŷ` is `[0, 0, 1]`. Its dot with the anchor is that anchor's own `z`,
so the offset is a stored number and **nothing rounds on the way in**. It is
also not a new normal invented for this pair: it is the one
`through_a_point_square_to` already ranks first among its three fallbacks here,
and `normal · up` comes to `1.000000`, so §164's 55° lean costs this pair
nothing. The constructor skips it only because the asked-for direction happens
to be exactly square as well, and "nothing beats the direction that was asked
for" then takes it unread.

### 183.4 Three things have to be right, and the step is not one of them

The good normal is necessary and it is not sufficient. Taking the probe's rows
for `along × ŷ` apart gives three separate requirements, each demonstrated by a
row that fails on that one alone:

- **The normal must not absorb the answer.** `along × towards` is
  `[0, 3.705489715806365e21, 5.047645337156084e153]`, two components **132
  decades apart**. One `f64` offset holding their sum swallows the smaller
  whole, and the smaller is where the answer is. Row one of §183.3 gives that
  normal a perfect anchor and it is still short by `8.19e4`.
- **The anchor must be a stored point.** `between_the_two_lines` computes
  `here + towards·share`, whose `z` comes out `41418752` against the thin axis'
  `41581179` — displaced by `162427`, the rounding
  `the_step_across_to_the_other_axis` leaves when it divides by `w·w` at
  `3.7e21`. With the big normal that displacement was absorbed by the offset and
  invisible; with a normal whose offset is exact it becomes the whole error.
  `midpoint/along × ŷ` stores `offset = 41418752` and misses by exactly that.
- **The middle must not be read as a difference.** `at_here + q·(at_there −
  at_here)` is right in exact arithmetic and useless in intervals when `q` is
  within `2.7e-265` of one and `at_here` is `3.7e21`: `q`'s own relative width
  of `2⁻⁵³` lands as `3.7e5` absolute. `there/along × ŷ` has the exact offset and
  the stored anchor and still refuses, on an enclosure `2.6e6` wide around a true
  middle of `9.98e-244`.

**The step's own rounding is not on this list, and §183.3's third row is why it
looked as though it were.** That row placed the middle *along the step*, which
is what the kernel did before §182. §182.4 already reads the picker at both
stored axes, so the step reaches the middle only through `apart²` — a relative
change of `1e-281` here. Written as one exact polynomial the middle comes to
`9.984566e-244` from the stored step and `9.984566e-244` from the exact one:
the same sixteen digits. `scratchpad/middle_from_axes.py`.

### 183.5 The fix, named

Three changes, all inside `intersect.rs`, none touching a type:

1. **Prove the normal rather than assume it.** `two_lines_told_apart` already
   proves or refuses. So try the exactly-square candidates in turn and take the
   first that *proves* the separation, instead of building one and refusing when
   it does not. This is the same shape as `orient2d`'s power-of-two retry: each
   candidate is proved or not, and nothing is nudged.
2. **Anchor on a stored axis**, not on a computed midpoint. §164 anchored between
   the lines to kill the `across·sin θ` term a leaning fallback leaves, and that
   reasoning is untouched for pairs near the origin — so this is a third
   candidate to try, not a replacement.
3. **Read the middle as one exact polynomial over one division.**
   `(at_here·2·apart² + (at_there − at_here)·(apart² + r₁² − r₂²)) / (2·apart²)`
   is a polynomial in stored numbers over one divide, which is §93's technique
   and the one §29.8 says is still untried on the other half of this row.

`HalfSpace` keeps its four numbers, its serde `[f64; 4]`, and every predicate
certified about it. §182.6's "still not excluded, still not built" is withdrawn:
it is now excluded, and three smaller things are built instead.

### 183.6 What this owes, and what it does not claim

**It owes a lane**, on §140.7's reasoning, unchanged. What the lane must not skip:

- **§182.5's seven.** All seven `crash-*` artifacts under `where_two_surfaces_cross`
  come back clean today and they do not come back the same way. A change that has
  gone conservative takes all of them; a change that has gone permissive is worse.
  Re-run the seven.
- **§140's pair and the thirty swept directions.**
  `a_thin_pipe_on_a_fat_pipes_skin_crosses_it_twice_either_way_round` and
  `where_two_side_by_side_pipes_stop_being_told_apart` are what a retry loop
  could quietly change, and §164's lean analysis is what it could invalidate.
- **The published refusal rates.** `how_often_naming_a_crossing_refuses` should
  move *down*. The skill's rule is that a rate going down is a result to report,
  not a silent improvement.
- **The regression test.** §182's asserts the two orders *agree*, deliberately
  not that they answer. If the lane makes them answer, that test's own doc
  comment is what has to be rewritten, not just its assertion.

**This does not claim the retry always finds a normal.** It claims one exists
for this pair, measured, and that the family §181.6 excluded was excluded on the
wrong ground. Whether three fallbacks are enough in general is the lane's
measurement to make, and if they are not, the honest answer is a refusal with
this section's number on it and not a fourth candidate picked to fit.

Commands this section was written from: `just decision 182`, `182.3`, `182.4`,
`182.6`, `181.6`, `181.7`, `181.2`, `164`, `29.8`, `39.4`, `43.6`; `just src
HalfSpace`, `through_a_point_square_to`, `two_lines_told_apart`,
`two_lines_between_two_pipes`, `between_the_two_lines`,
`the_step_across_to_the_other_axis`, `how_far_in_front`; a temporary probe in
`intersect.rs`'s own test module, reverted, kept at
`scratchpad/probe-182-anchors.diff`; and an exact rational evaluation of the
three pickers in `scratchpad/anchor_exact.py`.
