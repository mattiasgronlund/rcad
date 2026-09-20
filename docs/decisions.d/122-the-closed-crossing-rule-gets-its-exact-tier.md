## 122. The closed-crossing rule gets its exact tier, and the tier order is forced, fixed 2026-09-18

*§111.3 recorded a crossing whose sign exists and which the rule refused, called
it a gap rather than a limit, and left one design question live beside it: the
rule reads its second reference only on an exact `Decision::Zero`, so an exact
tier "has to decide which tier answers before it decides the sign". The tier is
built and the witness has its sign back. The ordering question has an answer, and
the answer is not a preference — there is an input on which the other ordering
gives a **wrong** answer rather than a slow one.*

### 122.1 What was built

`which_way_a_closed_crossing_turns` was one function and is now three:
`enclosed_which_way_a_closed_crossing_turns`, which is exactly the interval
arithmetic that was there before and is untouched;
`exactly_which_way_a_closed_crossing_turns`, which is the same rule over
`crate::exact::Exact` with no rounding anywhere in it; and the public function,
which runs the first and calls the second only where the first came back
[`Decision::Undecidable`]. That is the shape `orient3d` already has and §111.3
named as the one-member-wide family of fixes.

The exact level needed no new arithmetic. `Exact` already holds `of_each`,
`cross_of`, `dot_of_two_exact` and `sign`, so both references are four lines
each. The rule's latitude is written turned round — `longitude · (walked ×
gradient)` instead of `walked · (gradient × longitude)` — because `Exact::cross_of`
takes the stored triple on the right and a triple product does not care which of
its three vectors you start from. Exact arithmetic either way, so that is
spelling and not a second answer.

`the_exact_level_agrees_with_every_answer_the_enclosure_certifies` runs both
levels over every case in the file that the enclosure settles and requires them
to match. An enclosure is sound, so a disagreement would be the new level's bug;
this is the test that says the new level cannot quietly change an answer that was
already being given.

### 122.2 The ordering question, and the witness that settles it

§111.3 posed it as a sequencing problem — pick a tier, then pick a sign. It is
not one. **Which reference answers *is* a sign question**, and it is the same
sign question the tier is there to settle: the rule reads the latitude exactly
where the longitude's dot is zero, so "does the longitude answer" and "is the
longitude's dot nonzero" are one sentence written twice. An enclosure that
straddles zero has not said the dot is nonzero. It has said it does not know
which of the three it is.

So there were two designs:

1. **Level 2 picks the reference; level 3 finishes the sign of that one.**
2. **Level 3 re-asks both questions from the top**, longitude and then latitude,
   exactly as level 2 does.

The second is what is built, and the first is wrong. `Decision::Zero` out of this
rule means *the two surfaces touch rather than cross* and makes
`a_closed_crossing_is_walked_the_way_it_is_stored` answer `Ok(None)` — the caller
that writes an edge refuses and the caller that reads one back skips it. Design 1
reaches that answer off the longitude alone, which is the one thing §36.18's two
references exist to stop.

**The witness** is `a_longitude_that_is_exactly_zero_where_the_enclosure_cannot_see_it`,
and it is small enough to check by hand. With `c` the `f64` one step above one,
take `outward = [(1, 1, c), (1, 3, 2c)]`, `gradient = (1, 1, c)`, `axis = (0, 0, 1)`:

| | |
| --- | --- |
| `longitude = axis × gradient` | `(−1, 1, 0)`, exactly |
| `walked = outward[0] × outward[1]` | `(−c, −c, 2)`, exactly |
| `walked · longitude` | `c − c`, which is **exactly zero** |
| `walked · (gradient × longitude)` | `2c² + 4`, about `6.000000000000001`, so **`Decision::Pos`** |

And the enclosure cannot see that zero. `walked.x` is `1 × 2c − c × 3`, where
`2c` is exact and `3c` is not — `3c` is `3 + 1.5` ulp of `3` — so its interval is
an ulp wide. `walked.y` is `c × 1 − 1 × 2c`, both exact, so its interval is a
point. The dot's two live terms are `−1 × walked.x` and `1 × walked.y`, so their
difference is an ulp wide with the true zero inside it, and level 2 answers
`Undecidable`.

Design 1 would then ask level 3 for the longitude's sign, get `Zero`, and hand
back `Zero` — claiming two surfaces touch, for a wire that has a perfectly good
direction. Design 2 asks both and answers `Pos`. **So the ordering is forced, and
the sentence §111.3 left open is answered by deleting its premise**: nothing
decides which tier answers before the sign, because that decision is a sign.

What design 2 costs is one extra exact dot product, on a path that has already
given up once. Level 2 keeps its place at the front for the ordinary reason —
it is cheap, and where it commits it is right.

### 122.3 What moved, measured

**The witness in `rcad-geom`.**
`a_crossing_barely_one_f64_step_wide_has_no_sign_the_enclosure_can_prove` is now
`a_crossing_barely_one_f64_step_wide_is_settled_by_the_exact_level`. It answered
`Decision::Undecidable { enclosure: 1.1102230246251568e-17 }` and answers
`Decision::Pos`; the interval level, called directly, still refuses, and the test
asserts both so the second level is not quietly credited with the first level's
work.

**The witness in `rcad-topo`.**
`a_crossing_barely_one_f64_step_wide_is_refused_although_its_sign_exists` is now
`..._is_settled_by_the_exact_level`. It answered
`Err(Error::CannotTellWhichWayAClosedEdgeRuns { .. })` and answers
`Ok(Some(false))` — **which is exactly what `fbab36ff` answered**, as §111.3
predicted it would.

**A correction to §111.3, and it is only bookkeeping.** §111.3 writes as though
there were one witness and quotes one exact dot, `-6.93889390390722838e-18`.
There are two tests and they are two different questions. The `rcad-topo` one
takes its gradient and axis from a unit ball at the origin and the plane `z = 0`
at the corner `(0.1, 0.2, 0.3)`, and that is the instance the negative dot and
`Ok(Some(false))` belong to. The `rcad-geom` one hands the predicate
`gradient = (0.1, 0.2, 0.3)` and `axis = (1, 0, 0)` directly, which makes the
longitude `(0, −0.3, 0.2)` and the exact dot `+4.163336342344337e-18` —
positive, and so `Decision::Pos`. Both are right. The cross product
`(2.7755575615628914e-17, -1.3877787807814457e-17, 0)` is shared and is what
§111.3 was really describing.

**The corpus did not move, and that is measured rather than repeated.**
`just t how_often_a_real_file_imports --no-capture` on this tree prints
`nist: 1 / 44 imported = 2.3%` and `kicad: 114 / 323 imported = 35.3%`. §111.3's
`kicad: 97 / 323 = 30.0%` was true when §111.3 was written and is three lanes
behind now — the closed-edge lane took it to 110, the equal-pipes lane to 112 and
the homogeneous-cylinder lane to 114, all recorded on the pinned list in
`how_often_a_real_file_imports.rs`. A rate in prose goes stale by somebody else's
work, which is why §44.10 pins the files by name instead. What actually holds the
line here is `assert_pinned_kicad_imports`, which compares the 114 names and fails
both ways, and it passes — so no file changed side.

**`LED_D5.0mm-3.step` still refuses, with the same reason.**
`the builder or the sweep: the model is broken: a face walks a closed edge
against the way its surface winds`, read out of the same run. §111.2's point
stands untouched: that refusal blames the file for a direction the kernel
guessed, and it is not this rule's arithmetic that is wrong about it.

`just gate` is green on this tree at 1973 tests, which is `main`'s 1970 plus the
three this lane added.

### 122.4 What the exact level still refuses, and the gap that leaves

`Exact::times` declines as soon as one entry-by-entry product would round —
below `2^-969`, where the low half stops being a normal number, or above, where
the product overflows. So the third level can decline too, and when it does the
rule keeps the enclosure's refusal rather than inventing one.
`a_crossing_written_far_too_small_to_multiply_is_still_refused` is that case: the
witness crossing of §122.3 with both gradients scaled by `2^-600` and the axis
left where it was, which both levels give up on.

**That is a gap and not a limit, by §16.10's test, and it is deferred rather than
fixed.** The answer exists — it is the settled crossing multiplied by an exact
power of two, which cannot have changed its sign — and the family of fixes is
again one member wide and again the one `orient3d` already has: run both levels a
second time on a copy of the input scaled by exact powers of two. It is stronger
here than it is for `orient3d`, because both references are **multilinear** in
the four vectors — degree one in `outward[0]`, `outward[1]` and `axis`, and one
or two in `gradient` — so each of the four can be scaled by its own positive
factor and every scaling multiplies the answer by a positive number.

What that lane owes beyond the scaling itself is the demonstration of what is
*still* refused afterwards, which is a genuine spread: a vector whose own
components are so far apart that no single exponent brings them all into range at
once. §16.11 is the same distinction one predicate along. Not built here for
budget, and named rather than assumed — this section is not claiming the case is
unreachable.

