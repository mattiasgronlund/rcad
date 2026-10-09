## 349. Three surfaces written `1e-159` across meet after a slide by a power of two — 2026-09-28

*Queue row Q327, from §322.4 and §322.7's last bullet. Lane
`lane-q327-three-surfaces-slid-by-a-power-of-two`, off `612e1d5e`. §322.4 kept
the uniformly tiny triple refused and named the scaling family as the one to
answer it. **It does.** `meet_of_three_surfaces` now asks a refused triple a
second time, with every length multiplied by one power of two, and slides the
corners it finds back. Q100's seed is a named test, and its corners come back
equal, bit for bit, to the same three surfaces' corners at size one moved down
`2⁻⁵²⁸`. The row the seed came from refused 180 of its 2051 triples and now
refuses none, and that is pinned. What the slide cannot reach is a corner
among the subnormals, and a test demonstrates that as a limit.*

### 349.1 The refusal, reproduced

The seed `e0a2a205` in
`every_point_where_three_surfaces_meet_is_on_all_three_of_them.proptest-regressions`
is the planes `y = 2` and `z = 0` and a ball of radius three about `(0, 1, 0)`,
every number times `2⁻⁵²⁸`. It is now
`two_planes_and_a_ball_written_about_1e_159_across_meet_where_the_same_three_do_at_home`.
On the base it fails with the record §322.4 describes: `where three surfaces
meet`, level `Exact`, enclosure `inf`, which is `an_exact_step_could_not_be_taken`.
Every product of two lengths is near `2⁻¹⁰⁵⁶`. `Nearly` keeps each with a box
one smallest subnormal wide. Against a constant term of `−8·2⁻¹⁰⁵⁶`, that box
is far more than one rounding, so `is_within_a_rounding` refuses it.

### 349.2 What one slide does to each kind of surface

Scaling space by `2ᵏ` multiplies every **length** a surface stores by `2ᵏ` and
leaves everything else alone. Directions and normals say which way, and a
cone's slope is one length over another.

| Surface | Multiplied by `2ᵏ` | Left as its bits are | Built back with |
|---|---|---|---|
| Plane `n·p = d` | `d` | `n` | `Plane::moved_to` |
| Ball | centre, radius | — | `Sphere::new` |
| Pipe | axis origin, radius | axis direction | `Axis::moved_to`, `Cylinder::new` |
| Cone | apex | axis direction, slope | `Axis::moved_to`, `Cone::new` |
| Torus | centre, both radii | axis direction | `Axis::moved_to`, `Torus::new` |

**Every one of these stays exact while each length stays a normal number or
zero**, because a power of two changes only the exponent field there.
`slid_exactly` in `exact.rs` refuses any other result, and one refusal sinks the
slide. A subnormal slid *up* is exact too: it becomes a normal number with every
bit it had. A tube still thinner than its ring, and a radius still above zero,
come through an exact scaling unchanged, so no constructor can refuse what the
slide builds, and `the_same_surface_slid_by` panics if one does.

**The one trap is the direction.** `Axis::new` scales what it is handed to
length one, and a stored direction handed back to it can come out with other
bits: `Axis::moved_to`'s doc measures 24 750 of 100 000. That would be another
surface. So the plane keeps its normal through `Plane::moved_to`, and every axis
keeps its direction through `Axis::moved_to`.

The corners slide back by `2⁻ᵏ`. An end that slides up again to where it
started was moved exactly. An end that does not has landed among the
subnormals and been rounded, less than one subnormal off. It is stepped one
subnormal further out (`the_point_slid_back`). An end that runs off the top is
infinite, and a corner with an infinite side is refused. That is the same
reading `the_points_those_steps_name` gives a step past the top.

### 349.3 Which slide, and why centred

The first version brought the largest length to `[1, 2)`, the way
`exponent_that_brings_into_range` does for `orient3d`. The seed then passed,
but 52 of the `somewhere else` row's 2051 triples were still refused. A probe
that printed each refusal showed all 52 were shifted: places moved out by
`2^shift`, with sizes at `2^scale` far below that. A ball `2⁻⁶⁰⁰` across whose
places sit at `2¹²` has its radius squared at `2⁻¹²²⁴` once the `2¹²` is
brought to one.

What runs out is a product of two small lengths falling off the bottom, or of
two large ones running off the top. So the slide now **centres** the binades
the lengths span on one: `k = −(lowest + highest)/2`. That radius squared is
then `2⁻⁶¹²`, and both ends have the same room. For a triple of one size,
centring and the largest-to-one slide are the same slide to within a binade.

A probe that lifted the largest length to `2^L` instead, with `L` in 0, 100,
200, 250 and 300, left 52, 8, 0, 0 and 0 refused. Centring was taken over a
fixed high lift for a reason that was argued and not run. The route where a
plane meets two quadrics of one bend builds its line from the difference of the
two quadrics, so the line's point is degree two in the lengths and its divisor
degree one. A ball's `C` is then degree four and `B² − A·C` degree six. At
`2²⁰⁰` that is past `2¹⁰²³`, and the row draws no such triple to show it.

**Asked under `Policy::certified()` whatever the caller's policy was.** A
tolerant budget is a length at the size the caller wrote, and the slid triple
is another size. A tolerant policy only accepts more than a certified one, so
this can fail to give an answer but never gives a wrong one. That is the same
trade `whether_a_touch_is_on_it` makes. A certified answer logs nothing. The
records of the refusal where the triple stands are passed on only when the slid
triple refuses too, so an answered meet carries no records about questions its
answer did not rest on.

§322.4's gate is untouched. `Nearly::is_within_a_rounding` still refuses the
loose box where the triple stands, and it still guards every triple that no
slide moves. No tolerance was widened.

### 349.4 The oracle asks at size one, and why that is not easing it

The row's first run with the slide failed. The failure was not a loose box but
the oracle's own side question. The triple is `y = 2⁻⁵²⁸`, `z = 0`, and a ball of
radius `2⁻⁵²⁸` about the origin, which the line only touches. It meets in a box
of **no width at all**. That ball moved a millionth of its size clear is
`2·2⁻⁵²⁸·2⁻⁵⁴⁸` off in `‖p − c‖² − R²`, under the smallest subnormal, and
`Sphere::side_of` could not decide it. No box, however tight, gets that decided
where it stands.

So `on_all_three_and_clear_of_each_moved` slides the point and the three
surfaces together to about size one before asking either half. Scaling by a
power of two is the same question at another size. The box is slid outward
wherever an end would round, so it still holds the corner. The measured width
is still taken where the triple stands. The seed's named test passed both
halves where it stands before the oracle moved, so for the seed the slide
changes nothing.

`Sphere::side_of` refusing at `2⁻⁵²⁸` a question it decides at size one is the
same gap in another file (`sphere.rs`). It is not in this lane's files, so it
is named here and not touched.

### 349.5 What the slide cannot reach

**A corner among the subnormals.**
`three_surfaces_written_among_the_subnormals_meet_in_a_box_a_subnormal_step_or_two_wide`
writes the seed's triple `2⁻¹⁰⁶²` across. The slide up is exact and the corner
is found at size one. The slide back lands `x = ±√8·2⁻¹⁰⁶²` between two
subnormals `2⁻¹⁰⁷⁴` apart, and one such step is already `8e-5` of the ball.
The box that comes back holds the corner, moved down from the one at home, and
is at most two steps wide. Measured: `1.6e-4` of the shape. `√8` is not a
fraction, so no `f64` box of `x` there is narrower than one step. A box a
billionth of the shape, which is what the row publishes, does not exist in the
number type. **A limit** (§16.10), and the reason the generator's
`SMALLEST_SCALE` of `−620` is a sound place for the family to stop.

**Not claimed either way:** a triple whose lengths span more binades than any
one slide can fit, so that a product of the smallest pair falls off the bottom
while a product of the largest runs off the top. The row draws spreads up to
about 650 binades and refuses none of them. Nothing here tried wider, so
nothing here says whether that refusal would be a limit or a gap.

### 349.6 Measured

All on this lane's tree, `mise exec -- just t -p rcad-geom --test
every_point_where_three_surfaces_meet_is_on_all_three_of_them --no-capture`
unless named:

| `somewhere else` row, 2051 triples | refused | corners |
|---|---|---|
| no slide (mutant: `the_meet_slid_into_range(..).filter(\|_\| false)`) | 180 | 670 |
| largest length brought to one (probe) | 52 | 732 |
| centred, as committed | **0** | 806 |

- The widest box on the row is `3.02e-16` of the size with or without the
  slide. The other rows are unchanged: the control, leaning and turned rows
  refuse none and measure `2.16e-16`, `1.62e-14` and `2.00e-15`.
- Under the no-slide mutant, three tests fail: the seed's named test and the
  subnormal test with the record in §349.1, and the row on its new pin
  `measured.refused == 0`, with 180. Before the pin the row passed under the
  same mutant, because its refusals were only printed. The pin was not run
  against the largest-to-one slide; that slide's 52 was measured before the
  pin existed.
- `mise exec -- just t -p rcad-geom -p rcad-topo --no-fail-fast`: 2149 run,
  2149 passed, 11 skipped. No pin moved in either crate.

### 349.7 What this leaves

- `Sphere::side_of` refuses at `2⁻⁵²⁸` a question it decides at size one
  (§349.4). Whether the pipe's, cone's and torus's side predicates do the same
  was not run. The fix is the same slide in the predicate, and it is a queue
  row's to take.
- `where_a_line_meets_a_surface`, the ray caster's route, goes through the
  same `is_within_a_rounding` gate and does not slide. It was not tried on a
  tiny line and surface. It takes boxes as well as stored numbers, so a slide
  there would have to move boxes outward, as this one moves corners back.
- §322.4's last sentences and §322.7's last bullet say the uniformly tiny
  triple "refuses as it did". That was true when written and is not now. §322
  is not this lane's to edit, so the correction is this section.

### 349.8 A line and a surface written `1e-159` across meet after the same slide — 2026-10-09

*Queue row Q459, from §349.7's second bullet. Lane
`lane-q459-a-line-meets-a-tiny-surface`, off `2a06994c`, in two runs (§416).
`where_a_line_meets_a_surface` is the route `classify_point` takes to cast a
ray. Written `2⁻⁵²⁸` across, it refused a ball, a pipe, a cone and a ring that
it answers at size one, and so did `classify_point`. It now slides a refused
question the way §349.3 slides three surfaces, and both answer.*

**The refusal.** Each test in `where_a_line_meets_a_surface.rs` asks one
question at size one and again with every number times `2⁻⁵²⁸`, the line's point
and direction included, so the two are one question at two sizes. Before the
slide (`dcdf610b`) the tiny one refused for
`a_line_and_a_ball_written_about_1e_159_across_meet_where_they_do_at_size_one`
and the same test for a pipe, a cone and a ring, and for
`a_line_touching_a_ring_written_about_1e_159_across_touches_it_where_it_does_at_size_one`,
a line along the top of the tube that touches it at `±√15`. Each refused with
the record `where a line meets a surface`, level `Exact`, enclosure `inf`: a
product of two lengths there is near `2⁻¹⁰⁵⁶` and a ring's quartic near
`2⁻²¹¹²`, so an exact step could not be taken. Size one answers each, so each is
a **gap** (§16.10). The plane answered at both sizes and its test is a pin:
every product it takes is a length times a normal, near `2⁻⁵²⁸`.

**The slide.** As in §349.3: a question refused `Error::Undecidable` is asked
again with every length times one power of two, chosen so the binades the
lengths span are centred on one, under `Policy::certified()`. The records where
it stands are passed on only when the slid question refuses too. The lengths
are the surface's (`the_lengths_of`) and the ends of the line's two boxes.

**The line's direction is a length too, so it slides.** `2ᵏ·from + t·2ᵏ·along`
is `2ᵏ` times the point at `t`, so a step `t` is the same number at both sizes
and comes back as it was found, with nothing to move back. Sliding the point
alone would have scaled `t` by `2⁻ᵏ`. Each test compares the tiny steps with
the steps at size one **bit for bit**, which is as tight as a comparison gets:
the tiny box is not one ulp wider.

**A box slides exactly or the slide is not taken.** The row asked for the boxes
to be moved outward, as `meet_of_three_surfaces` moves its corners back. That
was not done. Each end of a box slides through `slid_exactly`, and one that
would not stay a normal number sinks the slide, as a surface's length does
(§349.2). A box rounded outward holds lines other than the one asked about. A
line that was exact, each box one `f64`, would then no longer be, and the exact
level could not run. A ring's quartic needs that level
(`WhereALineMeetsARingNeedsTheLineExactly`). Nothing that comes back is a place
in space, so no box has to move back.

**What does move back is a touch's reach.** `MeetsAt::MaybeATouch` carries how
far a ring's quartic reaches over the touch's box. That is a length to the
fourth, so it comes back slid by `2⁻⁴ᵏ`, and outward where it rounds
(`a_reach_slid_back`). Four times a slide can be more than the 2046 binades
`multiplied_by_a_power_of_two` covers in its two steps. `2f64.powi(-1030)` is
zero, so a slide down of 2060 binades puts both ends at zero, and one subnormal
out from zero, `2⁻¹⁰⁷⁴`, does not hold `2⁻¹⁰⁶⁰`. So it goes in steps of at most
a thousand binades, each a factor `f64` holds.
`a_reach_slid_down_past_2046_binades_still_holds_where_it_lands` pins that. At
`2⁻⁵²⁸` a length to the fourth is under the smallest subnormal, and the touch
test pins the reach at exactly `(−2⁻¹⁰⁷⁴, 2⁻¹⁰⁷⁴)`.

**A refusal by name passes through.** The first run threw the slid question's
error away. A ring asked about a line that is a box, `2⁻⁵²⁸` across, then came
back `Error::Undecidable`, because the quartic fell out of `f64` before the box
was looked at. At size one the same question is refused
`WhereALineMeetsARingNeedsTheLineExactly`. Now any refusal of the slid question
but `Undecidable` comes back as it is, since it is the question's at any size.
`a_torus_asked_about_a_line_that_is_a_box_1e_159_across_is_refused_by_the_same_name`
pins it. `classify.rs` reads both refusals of a torus the same way, as
`CannotFireARayAtAFacesSurface`, so this moves no answer there.

**Through the public API, the refusal reached `classify_point`.** A throwaway
test in `crates/rcad-topo/tests/` builds a ball of radius 3, a round bar of
radius 3 and height 4, and a cube of side 4. It classifies 11 points under
`Policy::certified()` at size one, then the same with every number times
`2⁻⁵²⁸`, and prints each answer and how many records it logged. It was not
committed. Before the slide, at `2⁻⁵²⁸`, 6 of the 8 points on the ball and bar
came back `Undecidable { enclosure: 0.0 }` with no records: four inside and
two outside. The cube's three points answered, and so did `(5, 0, 0)` beside
the ball and `(5, 0, 2)` beside the bar. With the slide, all 22 answer as at
size one, with no records.

The tiny refusals logged nothing. `classify.rs`'s ray arm `Err(_) =>
tally.grazed(0.0)` drops the records a refused ray cast carries, where the arm
for `what_this_face_holds` drains them into the log. That was read, not
traced. The slide answers these points, and no input on this branch
is known to reach that arm. It is a row's to find one.

### 349.9 Measured

- `mise exec -- just t -p rcad-geom --test where_a_line_meets_a_surface`: 21
  run, 21 passed on the branch tip. On `dcdf610b`, the tests before the slide,
  20 run, 15 passed, 5 failed: the ball, pipe, cone, ring and ring touch. On
  `cc613892`, the slide that threw the slid refusal away, 21 run, 20 passed, 1
  failed: the ring with a box line, left `Undecidable` and right
  `WhereALineMeetsARingNeedsTheLineExactly`.
- The first run measured two mutants. With `multiplied_by_a_power_of_two` in
  place of the thousand-binade steps, the unit test fails with `[-5e-324,
  5e-324] does not hold ±2^-1060`. With the reach left where it was worked out
  (`-4 * up` made `0`), the touch test fails with left
  `(-7.105427357601002e-15, 5.329070518200751e-15)`.
- The `classify_point` probe:
  `mise exec -- just t -p rcad-topo --test zz_probe_q459_tiny_classify
  --no-capture`, on `dcdf610b` and on the branch tip. Its lines read `PROBE
  size=1.1380524797363597e-159 ball r=3 point=[1.0, 0.0, 0.0]*size ->
  Undecidable { enclosure: 0.0 } records=0` before and `-> Inside records=0`
  after.

### 349.10 What this leaves

- §349.7's second bullet is answered: the tiny line and surface were tried,
  refused, and now meet where they do at size one.
- §349.7's first bullet, `Sphere::side_of` at `2⁻⁵²⁸`, is untouched here and is
  Q458's.
- `classify_point`'s ray arm drops a refused ray cast's records (§349.8). A
  queue row, filed with this section.
- **Not claimed either way**, as in §349.5: a line and a surface whose lengths,
  the line's direction among them, span more binades than one slide fits. A
  direction written at a scale far from the surface's widens the span. Nothing
  here tried it.
