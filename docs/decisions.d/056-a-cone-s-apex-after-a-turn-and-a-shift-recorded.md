## 56. A cone's apex after a turn and a shift, recorded 2026-09-06

Three things a consumer raised in a channel overnight, none of which was in any
rcad file when this session started. That is §55's defect, and the answer to it
is this section rather than another reply.

One was a defect that gave a consumer a confidently wrong answer, found by asking
and fixed here. One was a number nobody had measured, and measuring it says the
prediction was wrong by a factor of two — in rcad's favour, and for a reason worth
writing down. The third was fixed by the import lane in `8358620` while this was
being written; §56.6 says what that cost and what caught it.

### 56.1 The demand: exact equality against `Cone::apex()`

rscene shades a cone by asking the surface for a normal, and **a cone has no
normal at its apex.** So its apex triangle is special-cased on *exact* equality
against `Cone::apex()`, deliberately and not for want of a tolerance. A tolerance
there is worse than nothing: a mesh point a whisker off the apex sits on a
genuine ruling whose normal is a genuine answer, so widening the test hands the
wedge middle back for a point that had a right answer available. What the exact
test buys is that a miss is *visible* — the special case does not fire, and the
consumer can say so.

**A miss was therefore a wrong normal and not a refusal**, which is the failure
mode this kernel exists to avoid. rscene could not test it end to end: no
cone-faced solid can be built at their pin. So the equality was rcad's to
demonstrate or to deny, and nothing had checked it.

### 56.2 It was wrong, and only for a face that is both turned and shifted

The two numbers arrive by different routes, and the routes were not the same
movement.

- **The apex a consumer reads** is `FaceView::surface`, placement composed in:
  `Cone::transformed` carries the axis origin through `Isometry::apply_to_point`,
  once.
- **The apex the mesh emits** is worked out where the face is stored — §17.1's
  frame — because `the_surface_this_face_is_meshed_on` reads a placed face's
  surface out of the arena rather than moving it into the meshing frame and back.
  `Cone::point_at(u, 0)` there is the stored apex exactly: both terms it adds are
  zero vectors. That point was then carried out on
  `the_frame_this_face_is_meshed_in(face).inverse()` — and a placed face is meshed
  in its placement's own inverse, so the movement it rode was **the placement
  inverted twice**.

`Isometry::inverse` is not a bit-for-bit involution. It conjugates the
quaternion, which only flips signs, but it replaces the shift with `−R⁻¹t`, which
turns a vector and rounds; inverting again turns it back and rounds again. A pure
shift survives that — the turn is the identity and Rodrigues' form adds nothing —
and so does a pure turn about an axis through the origin, whose shift is zero and
stays zero. **A turn with a shift on it has nothing protecting it**, and that is
the ordinary case: it is what `Model::place_solid` is for and what an assembly is
made of.

### 56.3 The demonstration, and it is a census rather than one example

`a_placed_cones_apex_and_the_point_the_mesh_puts_there`, four tests over 105
placements of a pointed cone of slope a half and height eight.

| family | placements | the doubled inversion moves the apex on | apex missed, before the fix |
|---|---|---|---|
| no placement at all | 1 | 0 | 0 |
| a shift alone | 4 | 0 | 0 |
| a turn about an axis through the origin | 20 | 0 | 0 |
| a turn and then a shift | 80 | 50 | 50 |

The two right-hand columns are the same 50 placements. That correspondence is
what makes this a diagnosis rather than an observation: the misses are exactly
the placements the doubled inversion moves, and no others.

The third column is also what stops the suite passing for the wrong reason. Each
family measures what inverting its own placements twice *would* have cost, and
asserts the count against the sentences above — no pure shift, no pure turn, a
real share of the turns with shifts. Without it the file could go green over a
family that never exercised the bug and look identical.

Worst miss found: one ulp, `[3, 5, 7]` read as `[2.9999999999999996, 5, 7]`.

### 56.4 The fix

`back_out_of_the_frame_this_face_is_meshed_in` rides a placed point out on the
placement **itself**, which is the meshing frame's exact inverse, and falls back
to inverting the frame for an unplaced face — where the frame is
`a_shift_that_loses_nothing`, a negation, and exact either way. Five call sites in
`mesh.rs` took it.

It also makes an existing charge honest. `how_far_placing_this_face_rounds` bills
one `Isometry::POINT_ROUNDOFF` for carrying a mesh point out to where the solid
sits — one movement — and it is now one movement.

### 56.5 The floor at a distance is `18 · ε · L`, measured, and not `36 · ε · L`

rscene's rungs bracketed the floor of a radius-five ball at `1e14` only to
`(0.1732051, 1.7320508]`, and a prediction of **0.79936** had been written down
against it. Nothing had measured it, and `Error::DeflectionBelowWhatAMeshCanReach`
hands `closest` to callers — a published number with nothing checking it (§15.6).

Measured by `the_floor_under_a_placed_ball_is_one_placements_worth_of_rounding_and_not_two`,
at `1f7b75c`:

| the ball is out at | the floor | one placement's worth | ratio |
|---|---|---|---|
| `1e6` | `4.239223932641019e-9` | `3.996822872665007e-9` | 1.0606484 |
| `1e10` | `3.996827130754963e-5` | `3.996802890648965e-5` | 1.0000061 |
| `1e12` | `0.003996803131071608` | `0.0039968028886705476` | 1.000000061 |
| `1e14` | **`0.3996802891074774`** | `0.39968028886507634` | 1.0000000006 |

**So the floor at `1e14` is 0.39968, half the prediction.** It sits inside
rscene's bracket, which is why their ladder could not tell the two apart.

The remainder over one placement's worth is `2.4240105997601184e-10` at *every*
distance — the walk's own share, charged against how far the ball reaches in the
frame it is meshed in, and a placed ball is meshed in its own placement's inverse
so it sees a radius-five ball at the origin wherever the solid sits. A
distance-independent remainder is what pins the placement coefficient at one
rather than merely bounding it under two.

Said without leaning on the test having written the reach down right: the floor
**rises `3.9968028886505635e-15` per unit the ball is placed out, which is one
`Isometry::POINT_ROUNDOFF` to the last bit.** Taken as a slope off the two ends,
everything that does not move with the placement cancels, so that is the placement
term's coefficient on its own.

**Two changes would move these numbers and from outside the crate they are the
same shape.** A second placement term is one. The other is
`how_far_the_points_of_reach` no longer coming to `corners + radius` for a ball's
face — the §15.6 reach under-charge is a live change of exactly that kind — which
would leave the mesher right and this test's arithmetic stale. Both scale the
placement term by a constant, and no measurement of floors separates them. The
assertions say so and name where to look, because the first reading is the
alarming one and would otherwise be the only one offered.

**The number the refusal carries changed while this was being measured, and the
measurement did not move.** The deflection-floor unit landed in `8358620` between
the first run and this one: `room_a_mesh_has_to_work_in` now lets a caller's own
deflection through against `how_far_a_mesh_point_of_this_face_rounds` and
*reports* `the_finest_deflection_the_walk_will_take`, which adds what a step at
the walk's cap stands off its chord. That grew the remainder above by about eighty
— the first run read `3.031649005909485e-12` — and moved no ratio in the table,
because both numbers carry the same `how_far_moving_this_face_rounds` and the
placement term lives there. A test written against a recorded floor would have
gone red for a change that does not touch what it claims. This one is a closed
form and did not.

**Where 0.79936 came from, and it is not a defect in any doc comment.** It was
read out of `the_surface_this_face_is_meshed_on`, whose doc derives `36 · ε · L`
for a surface's movement round trip. That paragraph is the cost of a round trip
the mesher **does not make** — it is the argument for reading a placed face's
surface out of the arena — and it is demonstrated where it stands, by
`a_turned_curved_face_is_not_charged_for_a_round_trip_it_does_not_make`. Taking it
for a term in the floor is the misreading, not the sentence. The floor's terms are
the walk's and `how_far_placing_this_face_rounds`, and for an unplaced face
stored far out, `how_far_shifting_this_face_rounds` at
`SURFACE_AFTER_SHIFTING_ROUNDOFF = 8 · ε` — the 1.44 factor
`a_boolean_that_cuts_nothing_off_a_bar_does_not_make_it_much_coarser_to_mesh`
already measures.

A tolerant caller at `1e14` is cut at `ROOM_ABOVE_THE_FLOOR` times the floor,
which *is* 0.79936 as the floor stood at `c9ff754`. The number was right about the
wrong quantity.

### 56.6 The import census and colour — closed by another lane, and closed wider

The third channel item was `Import`'s doc saying **"what is walked past is counted
rather than forgotten"** in the same paragraph that lists styling among what is
walked past, with only the next clause narrowing the counting to shape-carrying
entities. `SHAPES_THIS_READER_DOES_NOT_TAKE` holds seven keywords and no styling,
so a `STYLED_ITEM` is read past and nothing on `Import` said so. A consumer read
the first sentence, took the census for a complete one, and was about to publish a
denominator saying the importer had dropped nothing out of 367 files — while it
had dropped every colour in all of them. §52.14's shape: a list sound about most
of its subject reading as complete.

**It is fixed in `8358620`, by the import lane and not here.** This session had
written the same fix independently and dropped it on finding the lane's, which is
wider in the part that matters: it also names what happens to a file that carries
styling and *no* solid. That file never reaches the map at all — `Import::read`
returns `TheFileHoldsNoSolid`, or `TheFileHasNoSize` ahead of it — so a caller
using the map as "what did the importer drop" gets an `Err` and never the empty
map it would have branched on. `styling_is_never_among_the_shapes_walked_past`
pins both halves over the real corpus, including the one NIST file with 272
`STYLED_ITEM`s and no solid.

Recorded here because the finding and the fix were two different sessions, and
§55 is about acknowledgements that live nowhere: the duplicate cost one test and
one doc edit, both thrown away, and finding the overlap took one `git diff`.

### 56.7 What this does not close

**Per-face colour out of STEP import is undecided, not refused.** rscene's §8.2
demand is well-formed — `STYLED_ITEM` / `COLOUR_RGB` /
`PRESENTATION_STYLE_ASSIGNMENT` / `SURFACE_STYLE_*` parsed, and a colour per
`FaceKey` on `Import` — and it was acknowledged twice in channel and written into
no rcad file until this line. It is recorded here as outstanding and nothing
more. rscene is **not blocked**: it took the refused path, draws flat and reports
colour unavailable, and its sink is built and tested, so adopting later is
source-side only.

**§56.5 measures a *placed* solid.** A solid whose stored numbers are far from the
origin — what a fuse bakes and what an importer writes — pays
`how_far_shifting_this_face_rounds` as well, and that case is pinned by the 1.44
measurement named above rather than here.

**The apex census sweeps placements, not cones.** One cone shape, 105 placements.
A cone whose stored apex is not at the origin would exercise
`a_shift_that_loses_nothing` on the unplaced path as well, and nothing here makes
one — but that is the **helper's** limit and not the kernel's. The suite goes
through `a_pointed_cone(model, slope, base_height)`, whose signature pins the apex
at the origin. `SolidBuilder::add_face` is public, is re-exported from `rcad`, and
takes an arbitrary `Surface`, so a caller puts the apex wherever it likes in one
line; `rcad-step`'s `cone_from` builds one from any file's placement. Unexercised,
then, and not unreachable.

**But the sweep that would exercise it cannot fail, and that is worth more than
the sweep.** `a_shift_that_loses_nothing` builds its run from the face's corners
*and* `placed_by`, which for a cone is `cone.apex()` itself. Each coordinate is
then either shifted by `low` — where Sterbenz makes `apex − low` exact, and adding
`low` back lands on a representable `apex` — or not shifted at all. There is no
third branch, so the apex comes back bit-for-bit for **any** stored apex, however
far out and however wide the spread. rscene measured one instance of that through
its own facade and got bit-equality; it had measured an identity.

So a census over stored apexes would be a green run over a theorem: an absence
that could not have been a presence proves nothing, which is the same reason
§42.12's tenth criterion makes the import rate carry a control. Written at all, it
needs the doubled inversion of §56.1 as its positive control, the way the 105
placements are the control on the placed side. **What deserves a test is one layer
down** — that the point the mesh emits
is a member of the run the shift is computed over. That is the invariant doing the
work, and it does not transfer: for a sphere `placed_by` is `ball.centre()`, which
the mesh never emits at all. Whether a ball's pole is exact for some other reason
is unmeasured and unasked.

**Nobody ships against any of this today, and the blocker is not the cone.** No
corpus file yields a cone face: 41 name `CONICAL_SURFACE` and none import, and a
census reading `FaceView::surface()` over the 98 files that do import finds 32 094
planes, 945 cylinders, one sphere and no cone. The 41 die upstream of the cone
across 24 unrelated reasons, overwhelmingly `B_SPLINE_CURVE_WITH_KNOTS` on an
edge; `CONICAL_SURFACE` itself maps fine. That day arrives when the B-spline edge
blocker clears, not on any cone work.

**Those four numbers are dated and nothing holds them.** They were taken at
`30d735b` by a throwaway probe walking `RCAD_STEP_CORPUS`, importing under
`Policy::certified()` and censusing `face.surface()` over `import.model().faces()`
— spelling-independent, with the `CONICAL_SURFACE` count a byte scan of the file
so it does not depend on the reader. The probe was deleted, and **no standing test
holds any of it**, so nothing goes red on the day they stop being true. That day
is a named event rather than a vague one: it is the day the B-spline edge blocker
clears, which is the same sentence two paragraphs up. Whether this should be a
standing test instead is §13's question and not settled here — the argument for it
is not the census but the tripwire, and the argument against is that it would be a
test whose subject is another milestone's blocker.

**How this paragraph came to be wrong is the part worth keeping.** It first said
"the public builder this suite goes through does not make one", which is a false
claim about the public builder dressed as a true one about a test helper, and two
readers took it for unreachability and planned around it. The correction then
overreached the other way — that an off-origin apex is "the normal case for
anything imported" — which was an unmeasured consequence asserted inside a
paragraph whose subject was a claim never checked against the public API. The same
shape twice in one thread, in both directions, caught both times by a consumer
reading it back rather than by anything in this tree. §51.10 is the control that
worked; the rule it enforces is CLAUDE.md's, that "no caller hits this" is checked
against the public API and not against today's callers.

