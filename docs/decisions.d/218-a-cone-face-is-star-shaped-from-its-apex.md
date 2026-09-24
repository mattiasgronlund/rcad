## 218. A cone face is one piece because it is star-shaped from its apex, the new run at the last two doors, and two loose threads pulled — 2026-09-24

§0's first row, closed — and closed by an argument rather than by arithmetic,
which is what §217.4 said the cost would be. Beside it the two doors §217.5
left, and the two things §217.3 recorded as found and did not diagnose, each
given a read-only lane. Four lanes off `bca11dd8`, two writing and each verified
by a Sonnet reader that reran every number, two read-only.

`just gate quiet` on the merged tree `f7da3c95`: **PASS, 2319 tests run, 2319
passed, 6 skipped**, 25 doc tests across 9 crates. The base `bca11dd8` was gated
first by the dispatching session rather than taken from §217's record of it —
PASS at 2314, and it matched.

### 218.1 A cone face is one piece because it is star-shaped from its apex

§216.5 sketched connectedness by walking the circle of rulings and settled a
cone cut by **exactly two** planes; §217.4 then counted the population and found
three planes every time and two not once, so the sketch proved a family that
holds none of the refusals. What landed is a different argument, and counting
the cuts is exactly what it does not do:

Every point of one nappe but the apex is on exactly one ray from the apex, and
the whole segment from the apex out to that point is on that same ray, so it
never leaves the nappe. A plane's signed value along such a ray is **affine** in
how far along it you have gone. So "in front of this cut" is a half-interval of
the ray, and "in front of every cut" is the intersection of half-intervals —
**one interval, however many cuts there are.** A ruling cannot leave the region
and come back, and that strip entered twice is precisely what §217.4 worried
three planes could leave. Three is settled the same way thirty is.

Ask on top of that that the **apex** be on or in front of every cut, and every
one of those intervals starts at the apex: the region then holds, with each of
its points, the whole segment back to the apex. Star-shaped from the apex, hence
one piece.

That second condition is a conjunct in the code and not a paragraph — the arm
reads one exact sign per cut, `flat.side_of_point(apex)` against `cut.kept` —
which is worth more than an argument that the population happens to satisfy it.
`is_a_family_this_follows`'s doc comment carries the argument where the next
reader meets it.

### 218.2 The nappe guard is not about connectedness, and the brief said it was

The dispatching brief handed the lane the star-shape hypothesis and told it to
check, not trust, two legs of it. One leg was wrong and the lane said so: **the
nappe guard is not needed for connectedness.** Both nappes' regions contain the
apex, so their union is connected too, and `is_a_family_this_follows` would pass
on it.

What the guard is for is the caller's *other* half — that the face is the
**whole** of the part in front of every cut. With the apex strictly in front of
every cut, the region on the double cone is a neighbourhood of the apex on both
nappes: two sheets pinched at one point, with the face one of them. So
`what_a_face_cut_by_part_of_a_crossing_holds` asks
`is_on_the_nappe_the_face_is_on` before it reads anything, and deleting that
guard costs a wrong answer rather than a refusal — `(0.5, 0, -1)` and
`(1, 0, -2)` come back **held** through the public `face_holds_the_point`.
That is §217.4's measurement, re-measured here rather than quoted, and the
correction is in the doc comment because it changes what deleting the guard
would cost.

**The brief was the dispatcher's, and it was wrong in the lane's favour.** Worth
saying plainly: a hypothesis handed down with two named legs to check is a good
shape for a brief, and the lane earned its keep by breaking one of them.

### 218.3 An apex behind a cut is a limit, and it has a ruling walk behind it

Where the apex is strictly behind a cut the region really can be two pieces, so
the refusal that stays is a **limit** and not a gap (§16.10). The demonstration
is exact and is a test rather than a sentence. On the cone of slope one about
`+z`, keep `x + z/√2 ≥ 1` and `−x + z/√2 ≥ 1`, apex a whole unit behind each.
Along the ruling at angle `φ` the two grow at `cos φ + 1/√2` and
`−cos φ + 1/√2`, and a cut the apex is behind is reachable along a ruling only
where its own rate is positive — so the arcs are `|φ| < 135°` and
`45° < φ < 315°`, which overlap in **two** arcs, `45°–135°` and `225°–315°`,
with `φ = 0` and `φ = 180°` carrying nothing.
`two_planes_the_apex_is_behind_leave_a_nappe_in_two_pieces` walks 3 600 rulings,
counts the disjoint runs that carry a point in front of both, and asserts
exactly two. Widening past it would want a predicate for whether the leftover is
connected, over where the cut planes meet on the cone — a part with no named
consumer (§13).

**And one correction to §217.4.** It says the three planes cutting the cone face
are the corner-up cube's *down-corner* three. They are the **upper** three: on
the side-6 cube centred on the apex the three lower planes sit at 54.7° to the
axis with the whole upper nappe strictly inside them, so they never cut the cone
at all. The apex is strictly inside all three of the upper ones, which is why
the conjuncted arm and §217.4's crude widening give the same 30 101. The
verifier reproduced that count independently — 152 answered cases in the
binary, the pointed-cone `Intersection` deciding 30 101, the four smallest at
9 326 and unmoved.

**The "arcs of circles for sides" clause is gone.** `is_a_family_this_follows`'s
first line promised it as a second thing checked there; no arm of it ever read a
side, and it was false of the arms it did have — an ellipse crossing has been
admitted since §211, and a cone face's own sides are conic arcs or rulings and
never circles. What the caller needs of a side is that `where_a_side_lies` can
place it, and `Lies::Across` is that refusal, read one side at a time.

### 218.4 The new run at the last two doors §217.5 named

Both closed, each with the test written first and watched fail.

`SolidBuilder::closes_on_one_edge` was a bare `==` against the old closed kind,
so a rim the fuse can now write could not be described again through
`SolidBuilder::add_wire`: fed back, it answered
`Err(ADraftWireIsTooShort { steps: 1 })`. §217.5 called this one line and it is;
what it was missing was the test, which is why §217.5 backed it out rather than
landing it.

`check_edge` asked the second surface a run names only under
`if let EdgeRun::PartOfWhereItCrosses`, so a closed rim's `and` was stored and
never looked at. Both kinds go through one block now, the closed kind handing in
an empty `picked_by` because it is the whole of a curve rather than a piece
somebody had to pick. The required finding, answered: widening it leaves every
rim the fuse actually writes valid — `a_rim_a_fuse_cut_round_one_cone.rs` sweeps
fused models carrying the new run through `Model::validate` and none moved.

**The carrier a test can reach that door with is a lie about its own shape, and
the test says so.** There is no in-crate builder for a solid whose wall is cut
in two — that shape comes out of the fuse — so the test swaps the run on a
round bar's cap circle, which is not really this kind of run at all: a cap
circle has a plane one side and a cylinder the other and they name it perfectly
well. It reaches `check_edges` because that is the sweep's third step and the
mesher is its sixth. The let-through twin therefore asks `check_edges` on its
own, because the whole sweep refuses that model for something else entirely:
`rcad-geom`'s `walk.rs` reads a closed rim as `[along[0], and]`, and `along[0]`
for the cap face is the cap's own plane, so it asks where a plane crosses itself
and gets `AWireDoesNotFanIntoALoopToMesh { TooFewCorners { count: 1 } }`.

### 218.5 Why two bars refuse differently depending on which is the tool

§217.3 recorded the asymmetry as found and guessed the pieces being placed are
the argument's faces one way round and the tool's the other. **Confirmed by a
run, and it is a gap rather than the draw.**

`choose` (`fuse/select.rs`) walks `scratch.pieces`, built arguments-then-tools,
and every `place_the_piece` is behind `?`, so the fuse reports the *first*
refusing piece — always an argument's, and the tool's are never reached. What
differs between the two orders is one number: how many corners the refusing cap
piece's wire has. In the pinned case both orders refuse over the **same region
of the same plane** — the two bottom caps are coplanar, the region is
`disc@0 ∩ disc@6` at `z = 0` — but as bar-at-0's cap piece it has three corners,
`(3,4,0)`, `(5,0,0)`, `(3,−4,0)`, and as bar-at-6's it has two, `(3,±4,0)`. A
bar's own rim start vertex is a corner of its own cap pieces and the other bar's
is not. Three corners is a triangle with an inside, so a candidate wins the
winding count and the refusal moves on to the next rule; two corners is no
polygon at all, so the winding is 0 for every candidate and the refusal is "no
point inside the piece".

It is a **gap** and not a limit: `(4, 0, 0)` is generated as a candidate for the
two-corner piece too and is strictly inside both circles on whole numbers
(`16 < 25`, `4 < 25`). The winding count throws it away. Not rounding either —
the split follows the bars at `x = ±3` and at `x = 100/106`, flips when the pair
is flipped, and follows the argument's `z` when they are staggered.

The cause is named in the code already: `in_the_bulge_of`'s `one_round_region`
admits a sphere or a closed cone and **not a cylinder**, its own doc comment
saying "not counted yet". The narrow fix is to admit a cylinder whose crossing
with the piece's plane closes, with the one-sided spelling living in `rcad-geom`
beside `Cone::is_cut_in_a_closed_curve_by` (§208.6, §23.8); the cone half of
this exact gap is already closed and tested, so there is a worked precedent.
**Fixing it will not make the bars fuse** — the order that gets a point still
dies at the next rule — so the test that fails today and passes after is the
asymmetry itself: the two orders reporting one refusal name. Its own doc comment
warns that counting the pipe turns two faces into four along one edge; that cost
is unmeasured prose and §217.2 and §181.6 are two fresh cases of such a cost
coming out wrong, so probe it before quoting a size.

### 218.6 A sphere crossed with itself divides by nothing, and a hair apart it answers wrongly

§217.3's other thread: why naming the cutting plane made a `Geometry(NotFinite)`
row start answering. **Answered with a run.** The cube's `x = 0` wall cuts the
ball in a circle and both faces either side of that rim are the one sphere. With
the old run, `walk.rs`'s `the_two_surfaces_it_runs_between` hands
`[Sphere, Sphere]` — the same sphere twice — to `where_two_balls_cut`, where
`apart = (0,0,0)`, `squared_gap = 0`, `along_the_centre_line = 0`, and
`part = 0 / (2 * 0)` is NaN. The midpoint is then NaN in all three coordinates
and `Plane::from_point_and_normal` refuses `NotFinite { what: "point on a
plane" }` — the exact string the old pin carried. §217.1's run makes the pair
`[Sphere, Plane]` and `where_a_plane_cuts_a_ball` answers with 9 points at
3.4e-13 deviation. §217.3's guess that `Arrangement::the_whole_surface` was
responsible is **wrong**; that call plants a finite centre.

**And the worse half, which nobody was looking for.** Two spheres that are the
same sphere written twice but *not bit-identical* — 1e-15 apart, §176.2's plane
case one surface up — answer **finitely and wrongly**: a great circle through
the ball's own centre instead of the real rim. The `along[0] == along[1]` guard
in `the_two_surfaces_it_runs_between` is bit equality and misses it. The new
run's arm is unconditional and ahead of that guard, which is exactly why a rim
the fuse writes is immune — and why nothing else is.

Four sites still hand a reader one point and no second surface, none of them
reached with a run here: `rcad-step`'s `round_run` returns the old variant for
every full CIRCLE or ELLIPSE whose ends are one vertex, without asking whether
the faces either side are the one surface, while the arc branch two lines down
already builds an `and`; `Model::add_closed_edge` hard-codes the old variant,
today used only for honest cylinder-and-plane cap circles; and
`where_two_balls_cut` itself has no guard for a zero gap — a guard refusing "these
two surfaces are one surface and name no crossing" would turn a NaN path into a
named limit and is a fix nobody has to reason about. The kept row asserts only
that two solids come back, so a regression writing the *right* variant with the
**wrong** `and` would give a finite circle in the wrong place and the row could
not see it: that is §216's 1.45 mm class one crate up.

### 218.7 What this leaves

The cone arm was §0's next unclaimed row and it is closed, with the proof rather
than the grid behind it. What §217.5 left is closed. What is newly open is
written above and none of it is in the tree yet:

- **The two spheres a hair apart answering finitely and wrongly** (218.6). A
  wrong answer outranks a refusal, and this one is measured.
- **The bulge that does not count a pipe** (218.5), a gap with a named narrow
  fix, a worked precedent, and the test that reaches it already written down.
- **`rcad-step`'s `round_run`**, the same drift at the importer's door.
- The four readers of the old closed kind that were found by grep and not
  audited — `mesh.rs`'s pole-insertion site among them, which matters because a
  cone has an apex.
