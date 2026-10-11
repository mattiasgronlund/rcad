## 35. Does a curved face hold this point, recorded 2026-09-01

Lane 3 of §31.6: §31.2's rows 10, 11, 12 and 15, written beside lane 2 and
against its absence rather than against its contract.

§31.10 calls row 10 the schedule risk of the whole milestone, says it has no
obvious route that avoids deciding on `(u, v)`, names three candidates, and says
**the lane's first day should be spent choosing rather than building**. It was.
§35.1 is the choice and the demonstration behind it, and the rest of this
section is what the choice cost and the four things nobody had noticed that fell
out of it.

### 35.1 The three routes, and the one that needed no curve followed

**Route 1, a ray *along the surface* from the point, crossing-counted against
the wires.** §31.10 already says it "turns the question into more three-surface
meets". It does, and the meets it turns into are the ones this milestone
**refuses**.

A ray along a ball's skin is the ball crossed with a plane through the point. A
boundary of a ball with a pipe through it — §31.7 criterion 13's own shape —
runs along where the ball crosses the pipe. So every crossing that ray has to
count is a point of ball, plane and pipe together, and
`meet_of_three_surfaces` refuses fewer than two planes by name (§34.10, first
row). `a_ray_along_a_balls_skin_needs_a_meet_of_three_that_this_milestone_refuses`
is that, with the same call and two planes in it as the control, because what is
refused has to be shown to be the degree and not the call.

**Corrected for Q1271.** Where the ball is centred on the pipe's axis, a plane
square to that axis, one holding it (Q1259) and one parallel to it and off it
(Q1271) are answered now, because the ball and the pipe share circles square
to the axis and each of those planes cuts them without a quartic. The
demonstration's first ray plane, `z = 0.5`, is the third kind, and is kept as a
control that is answered. Its ray plane is now `x + 16z = 8`, which leans to the
pipe's axis and cuts the pipe in an ellipse and the ball in a circle: that is
the quartic, and it is refused, with four points there to find. Nearly every
ray through a point is such a plane, so the route's verdict stands.

That is a demonstration and not a paragraph, which is what CLAUDE.md asks of a
claim that something cannot be done. **The family of fixes is named and it is
one**: an algebraic solver for a plane and two quadrics, which §31.2 already
defers to the milestone that builds one. Nothing about ordering the arithmetic
differently reaches it, because what is missing is not accuracy but a root of a
quartic.

Route 1 is also wrong for a second and independent reason, and it is worth
having because it would still hold if the algebra were free: §31.6 says lane 3
is written against lane 2's *absence*. A row 10 built on `meet_of_three_surfaces`
would have made the two lanes sequential and turned the milestone's only
concurrency into a queue.

**Route 3, cutting the face into pieces the flat argument does reach.** The cuts
have to be *placed*, and placing one on a curved face is where that cut's surface
meets the two the face's boundary already runs along — the same meet, arrived at
from the other side. It is route 1 with an extra step.

**Route 2 is what landed, and the precondition §31.10 gives it is not the
right one.** §31.10 says it "only decides for a face whose arcs all curve the
same way", which is a convexity condition and would rule out a ball with a pipe
through it. The condition that actually holds is topological:

> Every surface that trims a face meets that face's surface in **whole closed
> curves, all of which are edges of this face**.

Then that crossing cuts the surface into exactly **two** pieces, the face is one
of them, and which one is a sign — `side_of_surface`, the predicate that has
existed since M1. So the answer to row 10 is: for each surface trimming the
face, is the point on the side the face lies on. Nothing is constructed, no
curve is followed, nothing is ordered along one, no three surfaces are met at
once, and no `(u, v)` is computed anywhere.

**Two things are checked rather than assumed**, and each is one of the two ways
the answer would otherwise be silently wrong:

- Every trimming edge is a **whole closed curve** — its two ends are one corner.
  An edge with two different ends runs along a *piece* of a crossing, and which
  piece a point is on is row 8's question about where a curve runs.
  `Error::AFaceTrimmedByACurveThisCannotFollow`.
- The edges running along one surface are **every** curve the two surfaces cross
  in, counted by asking `intersect`, which is the one authority on that (§24.3).
  One of the two disks a pipe leaves in a ball's skin is bounded by a whole
  closed curve and would pass the first check; "the piece of the ball inside the
  pipe" names *both* disks, so a face holding one of two loops is refused.
  `Error::AFaceMissesPartOfWhereItsSurfacesCross`.

Both refusals are **gaps** and §35.11 names what closes each.

### 35.2 Which side a face lies on is a fact only its corners hold, and only topology can pick the corner

The route needs one number per trimming surface: which of its two sides the face
is on. **There is exactly one place in the model that says**, and finding that
out is the most useful thing in this section.

It is not the wire. §31.3 settles that a curved face's invariant is "every wire
keeps the material on its left", and reading a side off that needs the wire's
direction at a corner — which for a **closed** edge the model does not have.
§30.8 item 10 records that a closed edge that is a hole in a curved face has no
direction rule; §31.3 settles what the rule should be and hands it to lane 4.
Until lane 4 lands it, a closed edge's own sense is whatever `walk` produced for
the pair in the order it happened to be handed them.

**It is not the neighbouring face's own orientation either**, and that one is
worth writing out because it is the first thing to reach for and it is nearly
right. At an edge, the face on `S` and the face on `T` bound the same wedge of
material; write `a` and `b` for their two outward normals. Where the edge is
**convex** the material is `{d·a < 0} ∩ {d·b < 0}` and the `S` face carries on
into `d·b < 0`; where it is **reflex** the material is the union of the two and
the `S` face carries on into `d·b > 0`. So the answer is the neighbour's
orientation, turned round exactly when the edge is reflex — and `a` and `b`
alone cannot say which, because a dihedral angle and its explement leave the
same pair of normals. What tells them apart is the sign of the edge's own
direction against `a × b`, which is the direction the model does not have. The
two routes are therefore one route, and both wait on lane 4.

**And it is not the bounded-side rule that the flat path uses either**, which
is the third thing to reach for and the one worth being explicit about, because
the flat path really does use it. A closed curve cut on a *plane* by a ball or a
pipe bounds the part of the plane inside that ball or pipe — one of the two
sides is bounded and the other runs on for ever, so which is which is settled
without asking anything. On a **closed** surface both sides are bounded: a
circle on a sphere leaves two caps and neither is the outer one. So the rule
that rescues a round cap is silent about a disk cut out of a ball, which is
exactly the face `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` refuses.

So it is read off a **corner of the face that is off that surface**. Every
corner is either on the trimming surface — the corners of the very edges running
along it are — or off it on the side the face lies on. One corner off it
answers, every other corner off it has to agree, and a face with corners off it
on both sides is refused rather than guessed.

**And which corners are "on it" is topology and not arithmetic, which is the
regression this lane owns.** Asked with a predicate it works on a solid stored
where it sits and fails on one that has been *placed*:
`Isometry::apply_to_point` rounds, so the corner that is exactly on a cap circle
lands a hair off the cap's plane, on whichever side the rounding chose, and a
placed bar's wall reads as standing on both sides of its own cap.
`a_bar_and_a_ball_moved_and_turned_still_mesh_into_closed_surfaces` caught it
within a minute of the sweep's new check going in. The fix is to skip the
corners of the edges running along that surface **because they are on that
edge**, which no rounding takes away. §19.1 again, arriving through a door
nobody had opened.

### 35.3 Rows 11 and 12: the only thing in classification that is built rather than read

Where a straight ray meets a plane, a ball or a pipe is one equation in one
unknown of degree at most two. Put `x = from + t·along` into the very expression
`side_of_surface` decides the sign of — `|x − centre|² − radius²`, or
`|(x − origin) × direction|² − radius²` — and what comes out is `a·t² + b·t + k`
either way, the pipe being the ball with the axis crossed out of every vector
first. So the surface a root is worked out *from* and the surface it is later
tested *against* are the same one written the same way, which is §23.8's pattern
kept rather than found again.

Row 12 is `v · ∇S` at the root, times the face's own orientation. A polynomial
in the stored numbers with no normalization in it, exactly as §31.2 says.

Three things about the arithmetic, none of them guessed in advance:

**A point already on the surface is a root, and the quadratic formula loses it.**
With `k` exactly zero the two roots are `0` and `−b/a`. Read off
`(−b ± √(b²))/2a` instead, the first one comes back as a box straddling zero,
because enclosing a square root steps outwards — and then every point on a bar's
wall grazes its own face and `the_middle_of_a_bars_wall_is_on_its_skin` is
`Undecidable`. Factoring is not an optimisation here; it is the difference
between an answer and a refusal. This is `rcad-geom`'s own warning about
`root_enclosing` — "widening every root by an ulp would cost the exact cases
their exactness" (§29) — arriving one level up in a different shape.

**A division has to keep its exact cases too**, for the same reason: a ray that
starts exactly on a cap plane divides a numerator of nothing, and a widened
nothing is a box straddling zero, which reads as a ray that might be about to
cross the very face it is standing on. So each end of a quotient is multiplied
back out through `two_product` and stepped outwards only when that proves it was
inexact.

**`root_enclosing`, `quotient_or_everything`, `dot_enclosing`, `cross_enclosing`
and `gap_enclosing` are `pub(crate)` in `rcad-geom`, and this lane wrote all five
again in `rcad-topo`.** `rcad_predicates::Interval::divided_by` would have saved
one of them and is `pub(crate)` too — deliberately, and its doc says why:
dividing by a range that touches zero leaves an empty or unbounded interval and
would break `Interval::sign`'s no-empty invariant for anybody who could reach
it. So that one is not a missing export but a decision, and the copy here checks
its divisor first exactly as every caller inside that crate does. That is five spellings of five questions and §23.8 says
what that leads to. It is not fixed here because `rcad-geom` has one owner during
the fan (§15.9) and this lane is not it. **The fix is one line of
`rcad-geom/src/lib.rs`** — the module is already a module — and §35.10 hands it
to whoever opens that crate next.

### 35.4 A round cap fans into no triangles, and a round hole fans into a lie

`FaceView::for_each_triangle` fans each wire's corners from an apex. It is what
ray casting counted crossings with, what `box_around_its_corners` measured, and
what the planar volume sums are built on, and **it reads every wire as a
polygon**.

A round cap's wire is one closed edge and one corner. It fans into **no triangles
at all**, so before this lane a ray fired at a bar walked straight past both of
its ends and the count came back as though the bar were an endless pipe. Nothing
had noticed because `classify` refused a curved solid at the door.

A round **hole** is worse than invisible. Its one corner fans into a triangle
with no area, so the outer polygon's own fan covers the hole and a ray is counted
through material that is not there. Nothing builds such a face yet; a block with
a round hole through it is §31.7 criterion 11's headline shape and lane 5 makes
one.

**A flat face keeps the triangle route only while every side of it is straight**,
and that is what keeps a polyhedron's answer exactly what it was: every corner is
a coordinate the model holds, every test is an `orient3d` over four of them, and
nothing is built. Everything else — every curved face, and every flat face with a
curved side — goes by where the ray meets the surface, which is a division and
hands the predicate after it a box. The two routes are not two answers to one
question: the second is the general one and the first is the special case that
costs nothing.

**`for_each_triangle`'s other callers are still on the polygon reading**, and
§35.10 says which.

### 35.5 §25.9's box came due, and it is not the mesh that pays for it

`box_around_its_corners` says in its own doc that it is not a box round a curved
solid, and §25.9 hands the real one to "the lane that builds a mesh". A ray
caster needs it before that: the early "outside the box" test is a *proof* that a
point is outside the solid, and a box that misses the bulge turns it into a
confident wrong answer.

`SolidView::box_around_it` is that box and it needs no mesh. A ball's skin is
inside the box round its centre a radius each way, whatever the face is trimmed
down to. A pipe's is a radius from its axis, and how far *along* the axis a face
reaches is settled by its corners — the distance along an axis grows steadily
everywhere on a pipe, so it has no high or low spot in the middle of a face and
reaches its own furthest on the boundary. A flat face is the box round its own
corners, as before, so a polyhedron gets the very same box and the very same
three ray targets it got yesterday.

### 35.6 What `Containment::NeedsFlatFaces` became, and what the two `unreachable!()` are

`NeedsFlatFaces` said "this solid has a curved face and casting a ray at one is
not something this does yet". It is gone, and `classify` fires rays at curved
faces. What replaces it is narrower and it is not nothing:
`Containment::CannotFollowAFacesTrimming`, which says one face of the solid is
trimmed in a way §35.1's two checks will not follow.

**It is deliberately not `Undecidable`, and the reason is the one the old
variant's doc already gave**: `Undecidable` means every predicate was asked and
none could prove an answer, and this means the question was never put. Folding
the second into the first would let a caller loosen a policy and expect it to
help.

`select.rs:420` and `assemble.rs:943` justified an `unreachable!()` on "the fuse
refuses curved solids at entry", and lane 4 takes that gate off.
**Neither is a broken internal invariant, and both are refusals**, because what
reaches them is a face of a *caller's* solid:

- **`select.rs`'s `settle_against_each`** returns `Ok(None)`. A face nobody can
  follow is not a piece that could be placed somewhere else, and reading it as
  `Outside` would drop material. `Ok(None)` is what the function already means by
  "this piece could not be settled", so the fuse refuses rather than commits.
- **`assemble.rs`'s `one_shell_is_inside_another`** joins the arm that abandons a
  corner and tries the next. Saying anything stronger there would be a second
  opinion about which corners are worth trying; running out of corners is already
  `Error::Undecidable` and that is the honest end of it.

Both are edited here rather than left broken, because the branch has to compile;
they are lane 5's files and lane 5 should read them.

> **2026-10-04 note (Q504): running out of corners is no longer always
> `Undecidable`.** Folding a corner nobody can follow into the undecided ones made
> `one_shell_is_inside_another` report "undecided" for a question it never put,
> which is the mixing this section keeps apart. Since Q504 (316286ba), when no
> corner of a hollow was placed and one met a face nobody can follow, it is
> refused `Error::AFaceTrimmedByACurveThisCannotFollow` naming the caller's face
> that face was cut from; `Error::Undecidable` is left for the corners that were
> asked and not answered.
> `crates/rcad-topo/tests/a_shell_inside_another_names_its_refusal.rs` pins it.
>
> **2026-10-05 note (Q1007): since Q1002, the fuses that file pins now answer.**
> A flat face the cuts cannot follow winds its own chords and arcs instead of
> being handed to `one_shell_is_inside_another`'s abandon-the-corner arm, so the
> notched block's top is no longer such a face and every fuse in that file
> answers, each checked against its volume by hand. The file's name is now
> about what it used to pin rather than what it pins.
>
> **2026-10-06 note (Q883): each side is asked from the corner it starts at,
> and a ball's other pole is now reached.** `one_shell_is_inside_another` used
> to ask each side at `self.edges[side.edge].ends[0]`, the first end of its
> edge whichever way the side walks it, so a ball's one face — walked down its
> seam and back up it — had its north pole asked twice and its south pole
> never (16330bc9). Each side is now asked from `ends[!forward]`, the corner
> `Assembly::starts_at` reads off a dart, so every corner of every wire is
> asked. The `Error::Undecidable` record raised when every corner comes back
> `OnBoundary` or `Undecidable` now names the corners actually asked, not
> `self.corners[0]`, the whole answer's first corner and not a corner anyone
> asked about (28fa7550). And a corner whose `Containment` comes back
> `CannotFireARayAtAFacesSurface` — a ray that could not be fired at a ring
> face — is refused by the new `Error::CannotFireARayAtAFacesSurface`, naming
> the caller's face the ring face was cut from, rather than folded into
> `Undecidable` for want of a variant (b6e47fa6). No fuse is known to reach
> that last arm: every probe with a ring in it was refused earlier, at a
> curved-surface crossing, before `nest_the_shells` runs.

### 35.7 §31.10 item 5, scored: nothing this lane builds reaches `mesh.rs`'s two claims

§31.10 item 5 says a fused curved face is the first with a hole in it and that
`CurvedFaceWithAHoleGoesTooFarRound` and `MOST_PIECES_A_FACE_IS_CUT_INTO` both
rest on statements about today's builders. **Neither is reached by anything here,
and one of them is now known not to be reachable by the shape everyone expected
to reach it.**

`a_washer_passes_the_full_sweep` builds the first solid in this kernel with a
round hole in a **flat** face — a bar with a round hole straight through it,
which is §31.7 criterion 11's shape with a round outline instead of a square one
— and it passes `Model::validate`, mesher and all. `CurvedFaceWithAHoleGoesTooFarRound`
is about a hole in a **curved** face, and a washer has none: its two walls have
one wire each and its two caps are flat. So the claim at `error.rs:239` — "nothing
this kernel builds is such a face" — is untouched, and what falsifies it is a
curved face with a hole, which is a ball with a pipe through it and is lane 5's.

`MOST_PIECES_A_FACE_IS_CUT_INTO` is not reached either, for the same reason: a
washer's curved faces go right round and are cut into four pieces each, which is
what the constant's own doc predicts.

**The washer is the useful part of that test rather than the refusal it did not
provoke.** It is the shape lane 5 commits, built two milestones early, and it
says the sweep and the mesher are ready for it.

### 35.8 The numbers

Sixteen tests, and what each of §31.7's criteria in this lane's reach cost:

| | | |
|---|---|---|
| **9** | one authority on whether a curved face holds a point, agreeing with a fine mesh | met. `whether_a_curved_face_holds_a_point_agrees_with_a_mesh_of_the_same_face`, over a bar's wall and a ball's skin, 400 samples all compared and 304 of them held — the split derived rather than eyeballed |
| **10** | point in a curved solid, with the ON case | met. `the_middle_of_a_bars_wall_is_on_its_skin` and `a_point_a_hair_inside_a_balls_skin_is_inside_it`, both named in §31.7 word for word, plus five more that pin the cases they do not reach |
| **11** | `Containment::NeedsFlatFaces` gone | met for its half. `Error::FuseNeedsFlatFaces` is lane 4's and stands |
| **15 (row)** | a wire keeps the material on its left | **half met, and §35.10 says which half.** The sweep checks that a curved face does not stand on both sides of something trimming it; that it winds the right way round is not asked, because a closed edge has no direction to wind |
| **16** | a lattice curved control at 0% | met for classification. `a_grid_of_points_against_a_bar_and_a_ball_is_decided_and_records_nothing`: 9826 points, none undecided, nothing recorded |

Family by family and never blended (§17.7), over `Policy::certified()`:

| Family | points | undecided | could not follow a face | recorded |
|---|---|---|---|---|
| lattice bar | 4913 | 0.00% | 0.00% | none |
| lattice ball | 4913 | 0.00% | 0.00% | none |
| turned and moved bar | 2197 | 0.00% | 0.00% | none |
| turned and moved ball | 2197 | 0.00% | 0.00% | none |

**The turned rows recording nothing is structural rather than lucky**, and worth
saying because a zero that is luck is a zero that moves. The grid is drawn on the
lattice and the solid is not, so every ray meets a turned surface in a box; a
certified policy accepts nothing, so a predicate handed a box either proves its
sign and says nothing or gives up. A record there would mean a decision had been
taken on something nobody proved.

| | when this lane opened | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1009 | **1045** |
| of those, this lane's | — | **16** |
| suite duration | 96.5 s | 98.1 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 699 | 746 |

**Lanes 2 and 3 share a working tree, and the workspace counts cannot be split
by reading them.** §34.7 records 1045 as well, and says so in its own row: the
two lanes commit into the same branch from the same directory, so each has
swept the other's files into a commit at least once. Nothing was lost — the
later commit carries the later content — but a lane in this position should
stage by path rather than by `--all`, and this one learned that after two
commits rather than before.

The sixteen tests come to about nine seconds of processor time, seven and a half
of which is criterion 9's mesh oracle. The suite's own ninety-seven seconds are
still the one `rcad-geom` row §33.5 and §34.8 describe, and nothing here touches
it.

**One cost is not in that table and lane 5 should know it.** Asking whether a
face holds a point calls `intersect` once per surface trimming it, to count the
loops, and every one of those answers depends on the face alone rather than on
the point. A washer classified once is 2 walls × 2 roots × 3 rays = 12 curved
queries, each over 2 trimming planes, so **24 `intersect` calls for 4 distinct
answers**. It comes to about 90 µs per classification of a bar, which is nothing
for a ray caster and is not nothing for a fuse that classifies a witness point
per piece per pair. Nothing here has a target (§24.16), so it is a figure and not a
task; what would make it one is a curved fuse that spends measurable time in
`intersect` naming the same pair over and over, and the fix would be to ask once
per face rather than once per query.

### 35.9 Corrections to §31, §32 and §33

- **§31.10 item 1** gives route 2 the precondition "a face whose arcs all curve
  the same way". That is convexity, and it is neither necessary nor sufficient.
  What the route needs is that every surface trimming the face meets it in whole
  closed curves, all of which are edges of the face — §35.1. A ball with a pipe
  through it satisfies it and is not convex.
- **§31.10 item 1** also says there is "no such reference point off a sphere that
  works for the whole face". That is right and it is the whole reason the route
  that landed is not a crossing count: a crossing count needs a place whose
  answer is known, and a closed surface has none. §31.3's "a split curved face
  may have no outer wire" is the same fact seen from the topology side.
- **§31.10 item 2** guesses that the winding check "may turn out to be row 9's
  determinant at each corner, in which case it is cheap". It is not row 9's
  determinant, because row 9 needs the arc's direction and a closed edge has
  none. What is checkable without one is §35.2's side agreement, and it is cheap.
- **§31.2 row 10** leaves its polynomial and its degree unsettled with a "?".
  **There is no polynomial**, and that is the shape of the answer rather than a
  gap in it: the row is decided by other rows' predicates — `side_of_surface`,
  and `intersect` to count loops — and adds no arithmetic of its own. That is
  the whole reason it costs what it costs.
- **§31.2 row 11** marks its degrees 4 for a ball and 6 for a pipe as guesses.
  They are guesses about the *discriminant in the stored numbers* and this lane
  did not need one: what it solves is the quadratic in the ray's own parameter,
  which is degree **2** for both curved kinds and degree 1 for a plane, and the
  discriminant's sign is read as an interval and never expanded. So the row's
  number is neither confirmed nor corrected here, and the caller does not pay it.
- **§31.8 item 9** watches `MOST_POINTS_A_MESH_WILL_HOLD` for the first solid
  with five edges on a curved surface. A washer has six of those — four circles
  and two seams — and it meshes without coming near the cap.
- **§33.8's third trap** — a picker's `Undecidable` rules out nothing — did not
  fire, because nothing in this lane reads `picked_by`. It is still true and lane
  4 still meets it.

### 35.10 The traps for lanes 4 and 5

- **A closed edge still has no direction rule, and row 10 wants one.** §35.2 reads
  which side of a trimming surface a face lies on off a corner of the face that
  is off that surface, and a face whose *every* corner sits on its trimming has
  none — one of the two disks a pipe leaves in a ball's skin is exactly that.
  Once §31.3's rule for which of the two faces walks a closed edge forward is in
  the model, the side can be read off the wire instead and that refusal goes.
  **That is the single change that widens row 10 most**, and it is lane 4's.
- **`for_each_triangle` reads every wire as a polygon**, and the fan of a face
  with a curved side is wrong in two different ways (§35.4). Ray casting no longer
  uses it for such a face. **`measure`'s planar sums, `box_around_its_corners`
  and `bake.rs`'s box round a face still do**, and each is sound only while the
  faces it is asked about are polygons — which `measure` enforces with
  `is_wholly_flat` and `box_around_its_corners` says in its own name.
  **`bake.rs`'s `bake_one_face` has no such guard**, and lane 4 is what first
  hands it a curved face. Measured against a bar: the box it builds round the
  wall is the box round the seam, a line the wall bulges a whole radius out of,
  and the box round a round cap is `Bounds::nothing()`, because one corner fans
  into no triangles at all. A broad phase reading either will skip a pair that
  crosses. `SolidView::box_around_it` (§35.5) is the box to read instead.
- **The five enclosure helpers are written twice**, once `pub(crate)` in
  `rcad-geom` and once in `rcad-topo/src/classify.rs` (§35.3). Whoever opens
  `rcad-geom` next should export them and delete the copy; until then, a change to
  one is a change owed to the other.
- **A face trimmed by a *piece* of a crossing is refused, and lane 5 makes them.**
  A bar with a notch cut in its wall by a block is the first: its wall's boundary
  runs along the block's planes for a stretch and then stops. Row 10 refuses it by
  name, `classify` hands back `CannotFollowAFacesTrimming`, and the fuse refuses
  rather than guesses. §35.11 says what closes it.
- **The sweep's new check is only asked of a face whose every side is a whole
  closed crossing, and that is not timidity.** A face trimmed by a piece of a
  crossing straddles that piece's surface quite properly. A lane that widens the
  check without widening the condition will refuse sound solids.
- **`Containment` has a fifth variant and every `match` on it has to grow an
  arm.** §35.6 says what the two in the fuse became and why neither is a panic.
- **A round wire on a flat face takes its outer-or-hole sense from stored order,
  and nothing checks it.** `what_a_flat_face_holds` counts the first wire `+1`
  and the rest `−1`, which is `FaceData`'s "the first wire is the outer one by
  fiat" read literally — and `check_flat_face` deliberately walks past a wire
  that closes on one edge, because a circle has no polygon to wind. So a washer's
  cap arriving from a *file* with its two wires the other way round inverts
  silently, where the same mistake on a polygon is
  `Invalid::OuterWireWindsAgainstTheNormal`. It is a gap in the sweep and not in
  the answer, and closing it wants the bounded-side rule §35.2 rejects for curved
  faces and keeps for flat ones.
- **`SolidView::box_around_its_corners` now has no caller inside the kernel** —
  only two tests and a benchmark, all three of which want a bounding box and
  would be righter on `box_around_it`. Two public methods with "box" in the name
  is §23.8's shape reproduced rather than closed, and whoever is next in `view.rs`
  should move those three and delete it.
- **The fuse still winds a point round a piece's outline itself.**
  `fuse/select.rs`'s `winding_around` is a second answer to the question
  `holds.rs` exists to be the only one of, and it cannot simply call this one:
  its outlines are made of the `PredicatePoint`s the arrangement solved for, and
  the walk here takes corners the model holds. Widening the walk to boxes is what
  would close it, and it is lane 5's to weigh.

### 35.11 Every refusal this lane can produce, classified

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `AFaceTrimmedByACurveThisCannotFollow` | an edge of the face has two different ends, so it runs along part of a crossing | **gap** | `a_face_trimmed_by_part_of_a_curve_is_refused_by_name`. The point is on the piece or it is not; what says which is ordering points along a curve (row 8) after meeting three surfaces (row 4), and §34.10 files both. A closed-edge direction rule does **not** close this one |
| `AFaceMissesPartOfWhereItsSurfacesCross` | the face runs along some but not all of the curves its surface and a neighbour's cross in | **gap** | `a_face_along_one_of_two_loops_is_refused_rather_than_named_as_both`. What closes it is a way to tell one loop from another, which `Picker` already does for an edge (§33.1) — the same idea applied to a piece of a face |
| `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` | every corner of the face sits on the surface trimming it | **gap** | the side exists and the model does not record it; §35.10's first trap says what would |
| `Containment::CannotFollowAFacesTrimming` | any of the three above, met while counting a ray | **gap** | the three above, one layer up |
| `Invalid::AFaceStraddlesSomethingTrimmingIt` | a curved face stands on both sides of a whole crossing that bounds it | **not a refusal at all** — a model whose topology and geometry disagree, which arrives from a file. `a_curved_face_standing_on_both_sides_of_something_that_trims_it_is_caught` |
| `Undecidable` out of `face_holds_the_point` | a sign about a box could not be settled | **limit** where the box straddles the surface and a **gap** where the arithmetic ran out; no generator here reaches either, and §35.8's four families refuse none of it |

> **2026-10-11 note (Q1723): the `AFaceMissesPartOfWhereItsSurfacesCross` row's
> demonstration no longer exists.** Q1717 replaced
> `a_face_along_one_of_two_loops_is_refused_rather_than_named_as_both` in
> `crates/rcad-topo/src/holds.rs` with
> `each_patch_a_thin_bar_keeps_of_a_fat_bars_wall_holds_its_own_end_alone`, since
> two pipes crossing in two loops are read now. The refusal itself still fires:
> `crates/rcad-topo/tests/a_half_rings_flat_face_read_back_from_bytes.rs`'s
> `the_wall_holds_the_half_of_the_ring_its_wire_keeps` asks the wall of a
> half-ring whose wire keeps different sides of `z = 0` at its two circles, and
> gets `Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face })` naming that
> wall, for every point it tries.

