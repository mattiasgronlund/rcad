## 111. The sign §109.7 found was earned, and the numbers it was earned about were not, measured 2026-09-17

§109.7 said the closed-edge rule *"certifies a sign it has not earned"* and read
that as an arithmetic fault: a naked `> 0.0` over a cross product of squared norm
`2.55e-29`. The lane that went to fix it measured the chain and found the
arithmetic innocent. Every figure below was measured in this lane;
`a_hemisphere_sitting_on_a_pipe_of_the_same_radius_barely_crosses_it_at_all` in
`crates/rcad-topo/src/validate.rs` is the measurement, run without the corpus.

### 111.1 The file's own numbers, and why they reproduce off the corpus

`kicad/LED_D5.0mm-3.step` is a dome on a body of the same radius. It writes

```text
#39 = SPHERICAL_SURFACE('',#40,2.5);      #41 = (2.54, -5.051514762044E-015, 9.1)
#73 = CYLINDRICAL_SURFACE('',#74,2.5);    #75 = (2.54, -3.030908857227E-015, 9.1)
#76 = DIRECTION('',(-0.E+000, 3.330669073875E-016, -1.));
#22 = VERTEX_POINT('',#23);               #23 = (5.04, -5.051514762044E-015, 9.1)
```

Both radii are `2.5`, so the two are meant to be **exactly tangent** along the
equator and the file's own rounding — an axis three femtometres out and a centre
two femtometres off it — is the whole of what makes them cross at all. Four
CARTESIAN_POINTs and a DIRECTION is the entire input, which is why the test is a
unit test in `rcad-topo` and not a corpus test: the corpus is where the shape was
*found*, not what it needs to be re-measured.

At `#22` the rule computes

```text
∇S = (2.5, 0, 0)
∇T = (2.5, -2.020605904817e-15, -6.729969597663193e-31)
∇S × ∇T = (0, 1.682492399415798e-30, -5.0515147620424996e-15)
ẑ × ∇S   = (0, 2.5, 0)
(∇S × ∇T) · (ẑ × ∇S) = 4.206230998539495e-30
```

which is §109.7's two figures to the digit, independently arrived at. The
`‖∇S × ∇T‖²` over `‖∇S‖²‖∇T‖²` is `6.5e-31`: the two surfaces meet at `8.1e-16`
radians, under four `f64` steps of one.

### 111.2 The arithmetic is exact, so enclosing it changes nothing

**This is the finding, and it was a surprise.** Enclose that chain in interval
arithmetic over those numbers and it proves `Pos` with a width of about `1e-45`.
Read the five lines above and the reason is plain: every one of them is a single
product or a subtraction of numbers far apart, with nothing to cancel against.
The `4.2e-30` is not a rounding surviving a cancellation — it is the exact value
of the expression the stored numbers name. `> 0.0` gave the right answer.

So the fix §109.7 called for — *"a fresh interval predicate of the kind `march.rs`
already has"* — was tried, measured, and does not reach this input.

Going one step further and boxing the **gradients** as well, while keeping the
corner exact, does not reach it either. **That was reasoned here and then
measured by the wave-1 gate agent**, which built an outer enclosure with a
`nextafter` of slack per operation around §111.1's value and still got `Pos`
proved, no refusal. That enclosure is strictly wider than a tight one, so a
tight one refuses no sooner.

**The two exact edges this section first gave for that box do not belong
here, and are struck rather than corrected.** They were never checked into the
tree: the `nextafter`-slack procedure that produced them was run once by the
wave-1 gate agent and not committed, so nothing in `crates/`, `tooling/` or the
test suite can rebuild them — a search for the figures themselves and for
`nextafter` over all three turns up nothing. A bare number with no procedure
behind it to re-run is exactly the failure this file keeps having (see §124.1's
correction), so what stands is the qualitative result above — boxing the
gradients still proves `Pos` — and not the two figures that used to sit beside
it.

**The reason first written down here was wrong, and the correction is the
interesting part.** This section claimed `∇S` has no width because `at − centre`
subtracts exactly, and that `ẑ × ∇S` is then exactly `(0, 2.5, 0)` and multiplies
the one component carrying any width by exactly zero. Under boxing none of that
holds: `∇S` does carry width, `ẑ × ∇S` is not exact, and the surviving `y`
component is not multiplied by zero. The sign survives for a different reason —
**nothing in the chain cancels**. `(at − centre) · axis` is a single
non-cancelling product, so `∇T.z` is known to a relative rounding rather than an
absolute one, and the result sits about `2e14` of its own enclosure widths away
from zero. An enclosure only loses a sign where a subtraction eats the leading
digits, and there is no such subtraction anywhere in these five lines.

**Naming the family of fixes, per §16.10.** What is left is the inputs. `at` is a
decimal rounded to `f64` and is on neither surface; one `f64` step in its `y` is
`1.8e-15` against a `∇T · ŷ` of `-2.0e-15`, so moving the corner inside the box
it is known to turns the sign over. That is the same fact §39 already wrote down
for `not_provably_off`, which boxes a corner by
`Model::what_writing_a_point_down_costs` for exactly this reason. So the fix is
the corner arriving as a **box** and both gradients being worked out over it —
not a tolerance, and not a wider filter: the established width, already computed,
carried one function further.

### 111.3 What landed, and what separates the two shas

The rule's arithmetic is now `which_way_a_closed_crossing_turns` in
`crates/rcad-geom/src/which_way_a_closed_crossing_turns.rs`, answering with a
[`Decision`] and all four of its cases, and
`a_closed_crossing_is_walked_the_way_it_is_stored` matching them: a sign either
way, `Zero` for two surfaces that touch rather than cross, and
`Error::CannotTellWhichWayAClosedEdgeRuns` carrying the width that straddled.
CLAUDE.md's *"a predicate returns `Decision`, never a bare sign"* is now true of
this rule.

**What separates the two shas, measured both ways.**
`a_crossing_barely_one_f64_step_wide_is_refused_although_its_sign_exists` puts
`(0.1, 0.2, 0.3)` and `(0.3, 0.6, 0.9)` in as the two used gradients. On
`fbab36ff` the rule answers **`Ok(Some(false))`**; here it answers
`Err(Error::CannotTellWhichWayAClosedEdgeRuns { .. })`. The test was run on both
trees, the second in a detached worktree at `fbab36ff` with `is_err()` in place
of the variant the base does not have.

**That test was called `a_wire_that_is_only_the_rounding_in_its_own_cross_product_is_not_signed`,
and the name was a false claim.** The wave-1 gate agent caught it; this section
had said *"the same direction written twice, so their cross product is zero"* and
*"what comes out is the difference between two rounding errors"*. Both sentences
are wrong, and checking them in exact rational arithmetic over the stored doubles
takes a minute:

```text
0.2 == 2 × 0.1  exactly : True        0.3 == 3 × 0.1  exactly : False
0.6 == 2 × 0.3  exactly : True        0.9 == 9 × 0.1  exactly : False
```

So the third components are not proportional to the first two, the two triples
are **not parallel**, and their exact cross product is
`(2.7755575615628914e-17, -1.3877787807814457e-17, 0)` — the `z` exactly zero
because the doublings are exact, the other two genuinely nonzero. The crossing is
thin but **real**: `‖∇S × ∇T‖ / (‖∇S‖‖∇T‖)` is `7.39e-17`, about a third of an
`f64` step of one.

Two more things fall out, and both matter more than the name. The `f64` cross
product is **equal to the exact one componentwise** — each cancelling difference
is between two products within a factor of two of each other, so the subtraction
is exact and only the two product roundings are in it. And the exact dot is
`-6.93889390390722838e-18`, negative. **So `fbab36ff` was not certifying a sign
with nothing behind it. It was giving the right answer.**

#### Refusing this is a gap and not a limit, and that is left open on purpose

§16.10's test is whether the answer exists. It does: the sign of
`(∇S × ∇T) · (ẑ × ∇C)` is a polynomial in nine stored doubles, and this repo
already owns a level that always decides such a thing for exact `f64` inputs —
`rcad_predicates::expansion`, the cascade's third level. `which_way_a_closed_crossing_turns`
stops at the second. So the refusal is a **gap**, and the family of fixes is one
member wide: an exact-expansion tier under the interval one, the same shape
`orient3d` already has. It is not a wider filter and not a tolerance.

**Why it straddles is worth keeping too.** The true difference in the `y`
component is exactly `-1.000` ulp of the two products it comes from, and an
interval encloses each product to ±½ ulp — so the enclosure of the difference is
±1 ulp and the true value sits precisely on its edge. This is the cleanest
possible example of a second-level refusal that a third level settles.

**Not built here, deliberately.** A refusal is the safe answer and nothing is
wrong while it stands. But the record must not read as though refusing were
obviously right: it costs an answer that `fbab36ff` got, and the next lane
inherits a live question rather than a settled one. Note also that the second
tier (§36.6, §36.18 — latitude after longitude) still engages only on an exact
`Decision::Zero`, so an expansion tier has to decide which tier answers before it
decides the sign.

**It does not change what the LED file does**, for §111.2's reason. That file
still refuses, and the refusal is still the dishonest one: `the model is broken: a
face walks a closed edge against the way its surface winds`, which blames the
file for a direction the kernel guessed. `just t how_often_a_real_file_imports
--no-capture` names it, and the rates beside it were `kicad: 97 / 323 = 30.0%`
and `nist: 1 / 44 = 2.3%` — unmoved, which is the point.

### 111.4 The fuse propagates it, because a green gate is not a demonstration

`which_way_a_closed_edge_is_walked` in `crates/rcad-topo/src/fuse/assemble.rs`
ended `.expect("the curved check just above already ruled out the one case this
returns an error for")`. That sentence was written when the rule had one error in
it and stayed written down when a second arrived, so the change above turned a
refusal into a **panic** on the fuse's path.

The first draft of this entry argued the case away — *"nothing in the gate reaches
it, the fuse's own crossings are not this thin"*. That is the shape §16.10 names
and refuses: a claim about today's callers standing in for a claim about the
public API. A fuse's arguments are a caller's solids and a reader's file, and this
repo's rule is **panic on broken internal invariants, return `Err` on bad caller
input** — so an input that drives the rule to `Undecidable` there is data, not a
bug, and an `Err` is the only correct answer whether or not one is known.

So it propagates. The function returns `Result<Option<bool>>` and `Sides::of`,
which already returned a `Result` and already used `?` on the pole case one line
along, takes it with a `?`. No new error kind and no new refusal reason — the
`Error::CannotTellWhichWayAClosedEdgeRuns` the rule raises now travels out of the
fuse the way every other refusal does. The doc comment says what it errors with
instead of asserting it cannot.

**What is still outstanding** is §111.2: the corner arriving as a box. That
change needs both call sites, and `assemble.rs` is now written so that it costs a
third argument there rather than an unwinding of a panic.

