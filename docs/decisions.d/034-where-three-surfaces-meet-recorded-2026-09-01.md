## 34. Where three surfaces meet, recorded 2026-09-01

Lane 2 of §31.6: §31.2 rows 4, 5 and 8, one point on a closed crossing, and the
box §31.3 says a moved curved corner has to become. Written beside lane 3 and
touching only `crates/rcad-geom`, which is §15.9's rule in force.

Three of the five things this lane was told to build were as §31 describes them.
The other two were not, and both are recorded below rather than glossed: **the
degree §31.2 flagged as a guess is right for one of the two quadrics and four too
high for the other**, and **§31.3's "walk already starts a whole closed run at a
place that is a function of the pair alone" was false for two balls** — measured,
forty of forty swapped pairs walked into different points.

### 34.1 The degrees, measured against the guesses

§31.2 row 4 gives "3 / about 14" and marks the second as a guess. Both halves,
worked out term by term rather than estimated:

| The triple | The polynomial | Degree | §31.2 said |
|---|---|---|---|
| three planes | one 3×3 determinant of stored coefficients | **3** | 3, and it is right |
| two planes and a ball | `B² − A·C` for `A = D·D`, `B = V·D`, `C = V·V − m²R²` | **10** | about 14 |
| two planes and a pipe | the same with the axis crossed out of both: `A = ‖D × w‖²`, `B = (D × w)·(V × w)`, `C = ‖V × w‖² − m²r²` | **14** | about 14 |

`D` is `n₁ × n₂` at degree 2, `m` is the two-by-two minor of the normals that is
largest, and `V` is the line's own point offset to the quadric's centre or axis
origin, at degree 3.

**Four of the degrees the guess was carrying come from the point on the line, and
picking a different one is what spends them.** The obvious point on the line of
two planes is the foot of the perpendicular from the origin, and it is the one
§31.2's parenthetical is written about — it needs `‖n₁ × n₂‖²` underneath it and
carries four more degrees into everything built on it, which is exactly the
`|n₁ × n₂|²` the row says the discriminant has to be "cleared of". The point with
a **zero coordinate** — set `x_k = 0` for the axis whose minor is largest and
solve the remaining two-by-two by Cramer — needs no clearing at all, because the
factor was never introduced. Ball: 10 rather than 14. Pipe: 14 rather than 18.

So §31.2's number is right about the pipe and its parenthetical is a description
of the more expensive of two routes. §30.3's rule held again: the guess was
cheap to be wrong about and the first person who needed it did the sums.

**A degree is not what a caller pays, and the second number is measured too.**
`how_many_f64_the_discriminant_of_a_line_and_a_quadric_takes_to_hold` records how
many `f64` the exact value needs:

| | A | B | C | `B² − A·C` | spread |
|---|---|---|---|---|---|
| a ball, on the lattice | 3 | 3 | 4 | **5** | 166 |
| a pipe, on the lattice | 7 | 9 | 7 | **19** | 393 |
| a ball, turned | 9 | 11 | 16 | **36** | 516 |
| a pipe, turned | 14 | 19 | 20 | **57** | 761 |

`Exact::times` bounds a product at twice the two lengths multiplied, which for
the last row would be five figures; what keeps it at 57 is `expansion_sum`
dropping every roundoff that comes out exactly zero. **761 binary orders against
the 995 §32.1's window leaves** is the number to watch: the turned pipe row is
what runs out first if this degree ever goes up again.

### 34.2 `Measures` needed nothing, and `rcad-predicates` is still frozen

§31.5 weighed a `Measures` arm and said "`ManyLengths { how_many }` is parametric
and §31.2's highest row is a fourteenth-degree question, which it already
spells". **The fourteenth-degree question is an area.** A degree is a count of
stored numbers multiplied together and `Measures` is about *units*, and the two
part company here: the parameter `s` running along the line is dimensionless, so
the quadratic's `A` is dimensionless, `B` is a length and `C` is an area, and
`B² − A·C` is an area like every other discriminant in the intersector.

So the highest-degree question in the milestone files as `Measures::Area`, the
same arm `where_a_plane_and_a_ball_cross` has used since M3. §31.5 held, and it
held for a reason it did not have: **not because `ManyLengths` already spells
fourteen, but because fourteen was never the number that mattered.**

### 34.3 Two ways the boxes came out wrong, both caught by the oracle

Criterion 5 is `every_point_where_three_surfaces_meet_is_on_all_three_of_them`,
and writing it turned up two defects that reading the code did not. Both are the
same shape and it is §29's: **a difference of two large numbers, evaluated where
it cancels.**

**The discriminant's size was being read in intervals.** `B²` and `A·C` for a
line nearly tangent to a quadric are two large numbers that cancel to nothing, so
an interval evaluation of the difference reaches out to the rounding of the
larger, `√Δ` carries that, and so does every point built on it. Measured: the
somewhere-else row's widest box was **2.1e-8 of the shape**. Summed through the
expansion instead — the same difference, `B.times(&B).minus(&A.times(&C))`, with
no slide in it — it is **3.0e-16**.

That is §33.4's finding one question up, and it needs the distinction §32.1 drew
kept sharp: **`Exact::square_minus` is a sign and never a value**, because it
multiplies both sides by a positive power of two. It is what settles the sign
where the un-slid difference runs out of range, and it is never what the roots
are worked out from. The two are separate calls on separate lines.

**And `(−B ± √Δ)/A` is not good enough on its own.** A line a hair off parallel
to a pipe's axis has `A = ‖D × w‖²` down at `1e-32`, and then one of the two
roots is a difference of two nearly equal numbers divided by that. Measured on a
bar's own cap plane turned by a sixty-fourth: **a box fifteen hundred times the
shape**, for a corner that is perfectly well determined. The fix is the pair a
quadratic solver has used since Forsythe — `q = −(B + sign(B)·√Δ)` cancels
nothing, one root is `q/A` and the other is `C/q` — and because both forms are
valid enclosures of the same number, the narrower of the two is taken per root
and there is no branch to get wrong when `q` runs through zero.

Neither would have been found by reading. The first hid because the sign was
always right; the second hid because the case that shows it is a bar's own cap
plane, which is the most ordinary shape in the kernel.

### 34.4 A closed crossing that started in two places

§31.3 settles where the one vertex on a closed crossing goes: *"the vertex goes
where `walk` starts, because `walk` already starts a whole closed run at a place
that is a function of the pair alone"*. **That was true of three of the four
conic pairs and false of the fourth.**

`where_two_balls_cut` builds the radical plane through the gap between the two
centres, so the pair the other way round gets that plane reversed, a different
`Plane::basis` out of it, and a start a quarter of the way round the circle.
Measured over forty pairs: **forty of forty walked into different points**, and
not by a rounding — the first point was `(4.21, −2.44, −1.14)` one way and
`(1.90, 4.48, −1.14)` the other. The other three arms are order-free only by
accident of shape: both halves of their `match` call the same function with the
same arguments.

So `Surface` grows **one** fixed order over the numbers each kind stores, and the
two questions that need one share it rather than each inventing its own — §23.8's
pattern avoided rather than repeated. Sorting on stored numbers is sound for
exactly the reason §32.5's trap gives, that two faces which are the same plane
hold the same four numbers, and it is **not** sound for a `HalfSpace`, which
§33.8 says has no canonical form at all. Two pipes of exactly the same certified
radius were a tie in the same shape and are broken the same way.

`where_a_closed_crossing_starts` is the entry §31.3 asks for, and it is one small
function as that section predicted.

**The marched half still runs the other way round when the pair is swapped**, and
that is left alone. A conic is parametrized from a frame the unordered pair
fixes; a degree-four loop is followed along `∇S × ∇T`, which flips. §31.3 already
says which of the two faces walks a closed edge forward is decided by the
material on its left, so the direction flipping is the geometry rather than a
defect — but two halves of one function behaving differently is what a caller
assumes away, so
`a_conic_walks_the_same_way_round_whichever_face_asks_and_a_marched_loop_does_not`
says it out loud.

### 34.5 Row 8's other half, and why a conic needs no angle

The arrangement has ordered points along a straight crossing by its direction's
dominant axis since M1, which is the half §31.2 records as done. A closed
crossing has no axis to climb and no end to measure from.

**A conic is convex**, so three points on it run round it in the order they wind,
and which way they wind is the sign of the area they sweep. One determinant; no
angle constructed, no `atan2`, nothing projected into a surface's own two
coordinates — §15.5 kept, and §31.2 was right to be confident about this row.

Which way is forward is the crossing's own direction `∇S × ∇T`, so the answer
flips when the two surfaces are swapped. Working out which plane normal that
winds anticlockwise about is a line of algebra for each pair and it is written
down in the module rather than asserted. The pipe is the one worth having: the
ellipse's tangent is `n × ρ` for the outward radial direction `ρ`, and whether
that runs anticlockwise about `n` is the sign of `u · ρ` for `u` the step from
the ellipse's centre — which works out to `‖u × ŵ‖²`, above zero for every `u` in
a plane the axis is not parallel to.

### 34.6 How wide a moved curved corner's box is: five, not one or two

§31.10 item 6: *"whether that is one `Isometry::POINT_ROUNDOFF` or two — because
`Surface::transformed` rounds the surface as well as the point — is a derivation
nobody has done."* Done, and it is **five**, with `CORNER_AFTER_MOVING_ROUNDOFF`
published at `128 · f64::EPSILON`, about seven.

Two slips add. The point costs one `POINT_ROUNDOFF`. The surface costs one for a
ball — the centre moves and the radius is carried over exactly — and **four** for
a plane or a pipe, each derived separately in the constant's own doc: a plane
carries the turned normal's slip twice, once against the point and once through
the shift, and a pipe carries the moved origin, the tilt of the axis under a
point up to `2L` away, and the certified radius `r/‖w‖` moving with `‖w‖`.

**The half that matters is not the number.** `a_corner_read_out_of_a_model_and_moved_is_a_box_around_where_it_lands`
measures that **2265 of 3040** moved corners are certified `Pos` or `Neg` off a
surface they are a rounding from, when handed over as an exact triple. That is
§15.5's mistake arriving by §27.7's road, on the most ordinary shape in the
kernel, and if it had come out near zero there would have been nothing here worth
building. Handed over as the box, none of the 3040 is certified either way, and
the same box against the same surface moved comfortably clear is decided every
time. A search across sixteen orders of magnitude of size uses **a twenty-fifth**
of the published bound, which is the usual gap between a worst case and a drawn
one.

### 34.7 The numbers

Family by family and never blended (§17.7). 2048 triples per family,
`Policy::certified()`, two flat cuts and a third surface drawn flat as often as
round.

| Family | corners found | certified `Zero` on all three | refused | widest box, as a share of the size |
|---|---|---|---|---|
| lattice | 571 | 315 (55%) | **0.00%** | 2.2e-16 |
| leaning | 1281 | 0 | **0.00%** | 1.8e-14 |
| turned | 682 | 0 | **0.00%** | 2.4e-15 |
| written at `2^k`, moved by `2^m` | 646 | 240 (37%) | **10.83%** | 3.0e-16 |

**Criterion 6 is met and the lattice control is stronger than it was asked to
be.** §31.7 asks that a certified meeting of three lattice surfaces never refuse.
It never does — and 315 of its 571 corners come back as boxes of *no width at
all*, certified `Decision::Zero` on all three surfaces, which is criterion 5 met
to the letter rather than in the "never certified off" reading the other rows
need. Where a corner is a whole number the arithmetic rounds nothing and the box
is a point.

**The `2^k` row's 10.83% is the same refusal §32.1 and §33.5 both recorded and
neither moved**, arriving one question up: a product whose low half would land
among the subnormals. It is a **gap** and the same centred working frame §31.9
files as having no consumer is what reaches it. It now has a second one.

The intersector's own rates are unchanged by this lane, which is what they should
be — nothing here changes a sign question about a pair:

| Family | at §33.5 | now |
|---|---|---|
| lattice, planar / quartic | 0.00% | 0.00% |
| turned, planar | 0.10% | 0.10% |
| turned, quartic | 2.24% | 2.24% |
| leaning | 0.00% | 0.00% |
| the band | 3.76% | 3.76% |
| written at `2^k` | 3.95% | 3.95% |

| | at §33 | now |
|---|---|---|
| tests in `rcad-geom`, which is all this lane owns | 481 | **510** |
| tests (`cargo nextest`, whole workspace) | 1001 | **1045**, and still moving — lane 3 is committing beside this |
| suite duration | 94.4 s | 96.7 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 699 | **746** |

`just default` is green, `cargo-deny` included.

`just fuzz 180` ran 4 348 402 executions in 181 seconds, found **47 new seeds**
and **no crash**. Coverage went from 2505 to 2506 edges, so the new arithmetic is
reached by the corpus rather than sitting outside it.

### 34.8 The suite's ninety seconds are one draw, and splitting it does not help

§33.5 recorded the suite at 94.4 s with ninety of it in one test, and said the
honest fix was an optimised test profile rather than cutting draws. This lane was
asked to split that test's draws instead, so that the same 128 draws run on more
than one core. **It was built, measured, and reverted, because it buys nothing.**

The split was written so that the runner still samples the strategy all 128 times
in every one of the eight tests — the shard picks which draws are *walked* and
never which are *made*, so the union of the shards is the family draw for draw
and nothing is cut or moved. Measured: **draw 73 alone takes 88.6 of the 90
seconds** and every other shard finishes in under 0.4.

Draw 73 is two pipes down one direction, radii 5 and 4 with their axes 9 apart —
exactly tangent — with the second radius moved down one last bit and its axis
leaned by `2^-29`. Nearly parallel and nearly tangent leaves a very long thin
quartic loop, and the marcher walks 18 365 points round it at the deflection this
file asks for. That is the row working.

So no arrangement of the draws across tests can go below 88 s, and eight tests
that make the suite no faster are eight tests. **What does reach it is measured:
`[profile.test] opt-level = 2` in the workspace manifest takes `rcad-geom`'s
suite from 91 s to 2.7 s**, which is the same factor §33.5 measured for a release
build. That is one line, it changes no semantics — `debug-assertions` and
`overflow-checks` are separate settings and stay on — and it is **not taken
here**, because the workspace manifest is not this lane's and §33.5 already
called it a repo-wide tooling decision. Reported rather than taken, the way
§31.5 has a lane report rather than take a change to a frozen crate.

### 34.9 Corrections to §31, and what lanes 4 and 5 have to be told

- **§31.2 row 4's degree** is 10 for two planes and a ball and 14 for two planes
  and a pipe. The guess of "about 14" is right about the pipe; four of the
  degrees it carries for the ball are the foot-of-the-perpendicular point on the
  line, and the point with a zero coordinate never introduces them. §34.1.
- **§31.2 row 4's parenthetical** — "cleared of `‖n₁ × n₂‖²`" — describes the
  more expensive of two routes. Nothing is cleared, because nothing is
  introduced.
- **§31.5** keeps `Measures` unchanged for a reason it did not have. The
  fourteenth-degree question is an **area**: a degree counts stored numbers and
  `Measures` counts units, and the parameter running along the line is
  dimensionless. §34.2.
- **§31.3** says `walk` starts a whole closed run at a place that is a function
  of the pair alone. It did not, for two balls, and forty of forty swapped pairs
  proved it. Fixed rather than worked around. §34.4.
- **§31.7 criterion 5** asks for every meeting point to be "certified `Zero` on
  all three". Read literally that is unreachable for a box with width, for the
  reason `every_point_along_a_crossing_is_on_both_surfaces.rs` already argues one
  dimension up, so the oracle is M3's: never certified *off*, plus a control
  moved comfortably clear that must be decided. **On the lattice the literal
  reading is met as well**, 315 corners of 571.
- **`meet_of_three_planes` in `rcad-predicates` hands back a box that is not a
  place, and says `Ok`.** Cramer's rule there divides one interval determinant by
  another and checks neither for overflow, so three planes whose offsets are near
  the top of the range and whose normals lean at a small angle come back as
  `Ok(Constructed([…, […, −inf]]))` — a box a caller reads as a corner. Measured
  on `x = 1e300`, `y = 1e300` and a plane leaning by `2^-50`: the `z` side is
  `[−inf, −1.8e308]`. **This lane does not fix it, because §31.5 freezes that
  crate**, and reports it the way that section says a lane should. What lands in
  `rcad-geom` instead is that `meet_of_three_surfaces` reads such a box as the
  refusal it is;
  `three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere`
  is the demonstration, and the same three planes written lower down are placed,
  so it is a **gap**.
- **§31.7 criterion 8** names `three_points_on_a_conic_come_back_in_the_order_they_run_round_it`.
  It is `three_points_on_a_circle_come_back_in_the_order_they_run_round_it`, with
  one beside it per remaining conic pair — a bar's cap circle and the circle
  where two balls cross — because which normal a crossing winds anticlockwise
  about is a different line of algebra for each.

**The traps for lanes 4 and 5**, which build the curved arrangement and then
selection and commit on top of this:

- **`meet_of_three_surfaces` puts its three surfaces in a fixed order before it
  works anything out, and the points come back running along `n₁ × n₂` for the
  two planes in *that* order.** So `MeetPoint::OfThreeSurfaces`'s `which` counts
  the same way whichever face asked, which is what §31.3 needs it to do — but
  only while the caller hands over the same `Surface` values. Two faces on one
  plane hold the same four numbers (§32.5) and that is what the order rests on.
- **`Plane::meet_of_three` is still the one authority on where three planes
  meet**, and `meet_of_three_surfaces` calls it rather than answering a second
  time. What is new is that its refusal is *read*: a determinant worked out in
  intervals encloses zero whether it is zero or not, and the same determinant
  summed exactly says which. So three planes that share a line now come back
  `MoreThanAPoint`, three that share nothing come back `Nowhere`, and a corner
  Cramer's rule could not bound is placed off the line instead. **Lane 4 can
  switch `bake.rs` onto this and lose nothing**, because the planar path through
  it is bit-for-bit `Plane::meet_of_three`.
- **`MeetOfThree::MoreThanAPoint` is an answer and not a refusal.** Two faces on
  one plane with a curved surface across them share a whole curve, and so does a
  plane pair whose line runs down a pipe's own wall. A consumer that treats it as
  a failure will refuse a fuse that has no corner there rather than one that
  cannot be done.
- **A marched closed loop runs the other way round when its pair is swapped, and
  a conic does not.** §34.4. Whichever of the two a lane is holding, the
  direction to trust is the one §31.3 settles by the material on the left.
- **`which_comes_first_round_a_crossing` refuses a quartic**, and that refusal is
  a **gap**: the two loops are there and ordering points along one takes the
  marcher rather than a determinant. Lane 4 meets it the first time a corner
  lands on a ball-and-pipe crossing, which is a ball with a bar through it.
- **`where_a_corner_lands_after_moving` hands back a box and an `Approximation`,
  and the box is not what a vertex stores.** §31.8's prediction — that a mesh
  domain still holds no constructed corner after M4 — rests on `write_into`
  storing `corner.point`. The box is for asking predicates; the middle of it is
  for storing. A lane that stores the box has changed what a model is.
- **`Picker` still has no `transformed`**, unchanged from §33.8, and this lane
  did not need one. It comes due at lane 4 exactly as that section says.

### 34.10 What this lane leaves, classified

Every refusal `meet_of_three_surfaces` and `which_comes_first_round_a_crossing`
can produce, limit or gap by name (§16.10):

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` | fewer than two of the three are planes | **gap** | `three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved` exhibits `(3, 0, 4)` exactly on a pipe, two balls and a plane in whole numbers, and refuses two triples through it. §31.2 defers it and names the consumer |
| `Undecidable` out of `meet_of_three_surfaces` | a product the exact arithmetic needs falls out of `f64`'s range | **gap** | the `2^k` family's 10.83%; the same triples on the lattice refuse none of it |
| `Predicate(PlanesDoNotMeetInAPoint)` | three planes meet at a point neither Cramer's rule nor the line route can bound | **gap** | inherited from `meet_of_three_planes`, and now reached far less often, because the line route divides by a determinant known to full relative accuracy rather than by an interval one |
| `Undecidable` out of `meet_of_three_planes`'s own `Ok` | three planes meet past the top of `f64`'s range, and Cramer's rule says `Ok` holding a box with an infinite side | **gap** | `three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere`; the same three written lower down are placed |
| `TheseTwoSurfacesDoNotCrossInAConic` | two planes, or a plane down a pipe's own axis | **limit** | the question is a different one and the arrangement's dominant-axis rule answers it |
| `TheseTwoSurfacesDoNotCrossInAConic` | a ball against a pipe, or two pipes | **gap** | the curve is there and ordering points along it takes the marcher. **2026-10-09, Q1483:** closed for two pipes where the fat one swallows the thin one whole, with no marcher (§421) |
| `TheCrossingOfTheseTwoSurfacesIsNotClosed` | a crossing that runs on for ever | **limit** | the vertex does not exist: such an edge is cut by corners at both ends rather than closing on itself |

