## 109. The 44 were never classified, and the wall is not where it was written, corrected 2026-09-16

*§107.1's second row counts 44 files that stop on one refusal and nothing else.
The count is sound. Every verdict anyone attached to it was reached without
evidence, because the refusal names the whole solid and never the pair that
refused — so the tests that classified those 44 ran on zero pairs. Three
recovery lanes found this from three directions after the reboot of 2026-09-15.
Nothing here reopens §107.5's unit; it names what that unit has to build first,
which turns out to be the same thing two other questions were waiting on.*

### 109.1 The refusal names the whole solid, so the pair was never readable

`lane-which-pair-refused` ran the corpus three times and got the same answer
each time: of the 44 files, **0** publish an `#id` a pair could be read from and
**44** publish `TheSolidTheFileDescribesIsNotOne { solid: #15 }`. All 44 are
under `kicad/`.

The static route agrees without running anything.
`which_entity_the_builder_meant` (`brep.rs:902`) matches six `ADraft…` variants
and then `_ => return None`. The tangency refusal is a **unit variant** and
arrives as `Error::Geometry` out of `scratch.validate`, so it matches none of
the six; `what_stopped_the_build` (`brep.rs:727`) therefore takes the `None`
branch and publishes the whole-solid form instead. The entity in the message is
the solid, `#15`, in every one of the 44.

So any question of the shape "which two surfaces were too nearly parallel here"
was being asked of a message that does not carry the answer. **The census's
tangency test ran on zero pairs**, and every verdict it produced is a verdict
about nothing.

### 109.2 The honest counts, and the control nobody had run

`lane-near-parallel-census` graded its own inheritance and found the headline
wrong. `44 limit / 0 gap / 0 uncertain` becomes:

| was | is | what it means |
|---:|---:|---|
| 44 limit | **0 limit** | nothing here was shown to be beyond `f64` |
| 0 gap | **24 not attributed** | a tangency is somewhere in the file; nothing ties it to the refusal |
| 0 uncertain | **20 uncertain** | §109.3 — the pair named is not two surfaces |

The control is what settles it. "This file holds a tangency somewhere" fires on
**56.7%** of the 323 corpus files that do *not* carry this refusal, against
**54.5%** of the 44 that do. The criterion the old verdicts rested on
discriminates nothing at all — it is slightly *less* common among the files it
was supposed to explain than among the files it was not.

Two mechanisms behind that. `ProbeReport::reasons` is a `BTreeSet<String>` of
`kind_of` strings, and `kind_of`'s own doc comment says it never groups by `#id`
(`import.rs:288`), so the set records that a reason occurred and never where.
And the refusal has **six raise sites** in `src/` — `along_a_crossing.rs:326`
and `march.rs:286`, `:324`, `:328`, `:346`, `:416`, counted by reading them
rather than by grepping a total, since `along_a_crossing.rs:633` and
`march.rs:689`/`:712` are test assertions and not raises. Two of the six are not
about tangency at all: `march.rs:324` fires when `leaning.lo() <= 0.0`, the
enclosure of the cross-of-normals length merely touching zero, and `march.rs:416`
when `one_step` has exhausted its fifty shortenings. A file reaching either is
counted in the row with no tangency anywhere to find.

The lane also added an `assert!` so a census that matches **no** file fails
instead of printing a census of nothing, which is what its predecessor's first
run did.

### 109.3 Twenty of the forty-four are one cylinder split at a seam

Of the 44, **20** hold a pair that is coaxial, of equal radii, with an
axis-to-axis distance that prints `0.000000000000`. That is not two surfaces
tangent along a curve. It is **one** cylinder written as two `ADVANCED_FACE`s
either side of a seam, and a surface touches itself everywhere.

That is why those 20 are *uncertain* rather than *not attributed*: the question
"are these two surfaces too nearly parallel to follow" does not apply to them,
and what the kernel should do with a face pair that is one surface is a question
§107.5's unit does not currently ask.

### 109.4 What §107 said, and what it says now

§107.1's table stands. The counts of files were measured by `rcad_step::probe`
and nothing here touches them, nor does anything here touch §107.1's
co-occurrence finding — that the 44 never appear beside another reason — which
is a property the probe does measure.

What is withdrawn is every sentence that attached a *kind* to that row.
§107.5 calls the 44 "the biggest bucket that is a **gap** rather than a decision
to reopen". That may still be true. Nothing in the tree has shown it, and
§16.10 asks for a demonstration and not a paragraph — a rule that governs
calling something a gap exactly as it governs calling something a limit.
`44 limit / 0 gap / 0 uncertain`, wherever it was quoted, is superseded by
§109.2.

The cost of the old number is why this is a correction and not bookkeeping.
**`44 limit / 0 gap` was the number that said the near-parallel arithmetic was
not worth doing.** It does not say that any more, and nothing else currently
says it either.

### 109.5 The unit this cuts: a refusal that names its raise site and its pair

Everything above is one missing capability. A refusal carrying the two entities
it refused about, and the site that raised it, would have made §109.1
unnecessary, §109.2's verdicts readable, and §109.3 visible without a special
probe. It is also what §109.6 needs, which is why this is one unit and not two.

The shape, not yet built:

- the tangency refusal carries the pair it refused about, not the enclosing
  solid;
- it carries which of the six raise sites raised it, so `march.rs:324` and
  `:416` stop being counted as tangency;
- `which_entity_the_builder_meant` gains the arm that stops `brep.rs:727`
  falling back to the whole solid.

That is a change in `src/`, in `rcad-geom` and `rcad-step` both, which no
recovery lane owned. It is the first thing §107.5's unit should build, before
any arithmetic is touched, because until it exists the unit cannot tell whether
a fix worked.

### 109.6 The near-tangency wall is the stepping, not the cross of normals

`lane-near-parallel-arithmetic` confirmed its predecessor's *conclusion* and
demolished its *mechanism*.

The inherited doc comment said `march` asks `how_far_off_the_crossing` first, so
the cross-of-normals site can never hold a question it cannot answer. The test
behind that claim did show it — **over leans from `5.7e-14` down to `1.7e-18`, a
band nothing reachable presents.** Asked instead at `2.1e-8`, the tightest
crossing a ball on a pipe can actually express, one call at a time:
`how_far_off_the_crossing` answers `Ok(1.054e-8)` and does not refuse;
`which_way_the_crossing_runs` answers `Ok` and does not refuse; `one_step`
returns `Err(CannotFollowACrossingWhereItsSurfacesAreTangent)`.

So the wall is **`one_step` exhausting its fifty shortenings**
(`SHORTENINGS_BEFORE_GIVING_UP` is `50`, `march.rs:82`), eight powers of ten
above the refusal three lanes were pointed at. `one_step`'s own `# Errors` note
says that exhausting the budget "means the curve is not being followed rather
than that the budget was tight" — and that sentence is now the open question,
not the answer, because at a lean the geometry can actually present the other
two sites answer fine. §16.10 wants a demonstration for it either way. Today the
two walls raise the same message, which is why §109.5 comes first.

Two more defects in the same inheritance, both fixed:

- **A published bound with no test behind it** (§15.6). The doc comment
  published "a factor of one and a half" while the assertion said `< 30.0`,
  twenty times slacker. Measured worst is `1.37`; the assertion is now `< 1.5`.
- **A scaling claim that was prose only.** Scaling by three **rescues 171 of
  2 000 orientations and loses 64** — it trades refusals for wrong answers in
  both directions, which is worse than leaving it alone. §16.10's "name the
  family of fixes you considered" is what turned that sentence into a
  measurement.

The `1.11e-16` limit itself is sound and two-sided, each threshold falsified by
being put on the wrong side of itself.

Two gap-shaped hypotheses the lane could not test inside its remit, recorded so
the next one does not re-derive them:

- `settled`'s Newton correction `(s₁ − s₂c)/(1 − c²)` has `1 − c·c` reaching
  zero at a lean of `2.5e-8`, while `|u×v|²` still returns the truth to five
  digits at `4.9e-11`;
- `one_step`'s `turned >= 0.0` test may be flipping because the two crossing
  circles sit `4.2e-8` apart while a settled point there is known only to
  `1.05e-8`.

Neither is demonstrated. A lane at `one_step` should be briefed with both and
told to come back with a demonstration of gap or limit — after §109.5, so that
it can tell which site it is measuring.

### 109.7 A sign read off a vector of squared norm 2.55e-29

A separate fault in the same corpus, found twice over by two lanes that were
each diagnosing something else and had not seen each other's work. In
`crates/rcad-topo/src/validate.rs`,
`a_closed_crossing_is_walked_the_way_it_is_stored` reads a sign off a cross
product whose squared norm is `2.551780139113329e-29`:
`walked = [0, 1.682492399415798e-30, -5.0515147620424996e-15]`, and the sign
taken is that of `dot = 4.206230998539495e-30`. Both lanes reported those
figures to the digit. The shape is a hemisphere sitting on a cylinder of the
same radius and the same axis, exactly tangent, in `LED_D5.0mm-3.step` — one of
§107.1's 18 closed-edge files and the only one of them that is not the reader
defect §107.1's third row is otherwise made of.

**It is a gap and not a limit** (§16.10): the answer exists, and the arithmetic
certifies a sign it has not earned rather than refusing. The function returns
`Result<Option<bool>>`, so it has **no way to say it cannot tell** — `Ok(None)`
means the surfaces are tangent, `Ok(Some(bool))` means certified, and there is
no third answer. It takes `material_side: Decision` and its doc comment
carefully explains that `Zero` is filtered out and `Undecidable` becomes an
`Err` before it ever arrives — and then computes `against_the_frame > 0.0` off
an `f64`. Careful about the decision it is handed, careless about the one it
makes. Its second tier (§36.6, §36.18 — longitude then latitude) engages only
when the first is **exactly** `0.0`, and `4.2e-30` is not exactly zero.

What kind of fault it is was settled by experiment and not by reading. Edit the
STEP file so the cylinder is **exactly** coaxial with the sphere and re-import:
the refusal becomes `CannotFollowACrossingWhereItsSurfacesAreTangent` from
`along_a_crossing.rs:326`, where `Decision::Zero` refuses honestly. The real
file's axis is off by about `3e-15`, so it slips past that check as a genuine
but absurdly thin crossing and only then meets the naked sign read.

**Booked, not built, and deliberately so.** The first reading of this called the
fix small and self-contained; that was a guess, and the lane measured otherwise.
A certified sign over vectors that have *already* been through floating-point —
`gradient_of`, `the_way_it_is_parametrised_about` — is a different soundness
problem from `orient3d`'s exact inputs, because these inputs carry error before
the predicate starts. It needs enclosure-aware inputs or a fresh interval
predicate of the kind `march.rs` already has, plus an `# Errors` change on the
function, plus this entry. That is a lane of its own, owning `validate.rs`, an
`rcad-topo` test and the new predicate. One corpus file depends on it, so
nothing about the ordering is urgent.

