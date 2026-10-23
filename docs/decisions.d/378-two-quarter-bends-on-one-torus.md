## 378. Two quarter bends on one torus, end to end, under every rule — 2026-10-02

*Queue row Q754, which replaces Q198. This lane changed no source: it adds
`crates/rcad-topo/tests/two_quarter_bends_on_one_torus.rs` and this section.
Every number below was run on the base, `a189d01b`.*

In short: of the six fuses, four answer right and the two unions are refused
`ResultIsNotManifold { count: 4 }`. Selection keeps the right pieces. Assembly
then joins two different circles into one edge, because it names a closed
curve by its two surfaces alone, and a plane through a ring's axis cuts the
ring in **two** circles. That refusal is a gap: with the two circles kept
apart, in a scratch copy, both unions answer a half bend that validates and
measures `4π²`.

**Fixed by Q756, 378.6**: a closed crossing is now named by which of the
plane's two circles it is as well, and both unions answer the half bend. The
same fix answers all ten of §377's refused fuses.

### 378.1 The pair, and what each rule should give

Ring four, tube one, both bends about `+z`: one from `+x` round to `+y`, the
next from `+y` round to `−x`. `a_quarter_bend` and its two consts are copied
from `two_frustums_stacked_on_one_cone.rs`, whose test fuses the same two with
the second one's axis turned round (§357.4, Q513).

The two bends meet only at one cap, the disk on `x = 0` round `(0, 4, 0)`. The
first faces `−x` there and the second `+x`. So their insides are apart, and
what each rule should give needs no measuring:

- **union**: one half bend from `+x` round to `−x`. Its two caps both lie on
  `y = 0` and face `−y`, one round `(4, 0, 0)` and one round `(−4, 0, 0)`.
  Pappus: a quarter is `π·1² · (2π·4)/4 = 2π²`, so the half is `4π²`,
  `39.47841760435743`.
- **intersection**: two solids that share only a disk share no volume, so the
  answer is no solid at all. That is §191's rule for every pair whose insides
  are apart, and `Model::fuse`'s doc says an empty answer is a success.
- **difference**: nothing of the first bend is inside the second, so the
  answer is the first argument whole, `2π²`.

### 378.2 What each fuse gives

`mise exec -- just t -p rcad-topo --test merged -E 'test(/^two_quarter_bends_on_one_torus::/)'`,
three tests, each fusing both ways round, all passing:

| rule | `+x→+y` first | `+y→−x` first |
|---|---|---|
| union | `ResultIsNotManifold { count: 4 }` | `ResultIsNotManifold { count: 4 }` |
| intersection | no solid | no solid |
| difference | the `+x→+y` bend | the `+y→−x` bend |

Every answer comes with an empty log and passes `Model::validate`. Each
difference has the first bend's three surfaces and measures inside the bound
`measure` gives with it at deflection `0.01`: `19.706118053894524 ±
0.3921329444569594` and `19.706144105509548 ± 0.3921333372476393`, against
`2π² = 19.739208802178716`. The two volumes are from a probe test that
printed them, on the same base.

**Q198's refusal is not reached any more.** Q198 said the wall piece's points
on the inner equator got `CannotFireARayAtAFacesSurface` from the other bend's
torus face, before Q102 (`5862b49e`) and Q667 (`7c980d78`) landed. With a
print in `select::choose` (378.3 says where), every one of the six pieces in
all six fuses gets placed. Against its own group each is `Outside`. Against
the other group each wall and each cap on `y = 0` is `Outside`. The two caps
on `x = 0` are `On` each other, facing opposite ways, so union and
intersection drop both and difference keeps the argument's.

### 378.3 Where the union is refused, and why

The probes ran in a copy of the base exported with `git archive` into the
session's scratchpad. They are temporary `eprintln!`s, and none of them is in
this branch.

**Raised at** `Assembly::darts_along_each_edge` in
`crates/rcad-topo/src/fuse/assemble.rs`. That is the first step of
`join_up_what_is_one_face`, which `assemble` calls straight after
`Assembly::from_the_pieces`. A print of every edge's claimants there, union
with `+x→+y` first:

- the edge made from the first bend's start circle, on `y = 0` round
  `(4, 0, 0)`, is claimed by four faces: that bend's cap on `y = 0` and its
  wall, **and the second bend's cap on `y = 0` and its wall**;
- the edge made from the shared circle on `x = 0` is claimed by the two walls,
  which is right.

The other order is the same, with the two bends' roles swapped. So the second
bend's end circle, round `(−4, 0, 0)`, became the same edge as the first
bend's start circle, round `(4, 0, 0)`.

**Why.** The arrangement hands each wall back as a piece with two wires of one
side each, the two circles. The seam along the inner equator is gone, so the
corners `(3, 0, 0)` and `(−3, 0, 0)` each do nothing but start a closed curve.
`Corners::with_each_closed_curve_started_once` then groups every such corner
by the curve it closes, and names that curve by
`names_of_the_two_surfaces` alone: the plane `y = 0` and the ring. Both
circles get that one name. A print of its candidates there:

    curve [y = 0, the ring]: vertex 4 at (−3, 0, 0), vertex 1 at (3, 0, 0), each closing only this curve
    curve [x = 0, the ring]: vertex 2 at (0, 3, 0), vertex 3 at (0, 3, 0), each closing only this curve

The function keeps the first candidate and hands every other one that closes
only this curve over to it. So vertex 1 becomes vertex 4's corner. `Edges::of`
then names an edge by its two corners and, for a closed one,
`which_curve_it_is`, which is the same two surfaces again. Both circles come
out as one edge, with four faces along it.

The `x = 0` line is the joining this function was written for: two vertices
at one point on one circle.

**The counter-probe.** In the scratch copy only, `on_curve` was keyed by the
two surfaces and the vertex's exact coordinates, so the two circles stayed
apart. Nothing else changed. Both union orders then answer one solid of four
faces: the two caps on `y = 0` facing `−y` and the two walls. The log is
empty, `Model::validate` passes, and the solid measures
`39.41226215940417 ± 0.7842662817045987` at deflection `0.01`, against
`4π² = 39.47841760435743`. The union test of this file then fails, `first
quarter first true: Ok(1)`, and the other two still pass. That is what shows
the pin can move. Comparing coordinates is not the fix, as 378.4 says: it is
only the shortest way to show that nothing after the naming refuses.

### 378.4 Gap or limit, and the fixes considered

**The union is a gap** (§16.10). The answer exists. The model can hold it:
each circle keeps its own vertex, as each bend already stores it. And the
counter-probe reached it with the rest of the fuse unchanged. The
intersection and the difference answer right, so they are neither.

The names `assemble.rs` builds corners and edges from are made of three
things: the surfaces' bits, the coordinates, and the input's own vertices. So
there are three families of fix:

1. **Tell the two circles apart by coordinates**, as the counter-probe did.
   Rejected. The module doc of `assemble.rs` says no coordinate decides what
   is one corner, only a name does. Keyed by coordinates, two input vertices
   at different points on one circle would also stop being handed to one
   corner, which `with_each_closed_curve_started_once` does on purpose. No
   test of that case was run here.
2. **Never let one input vertex take over another.** Rejected for the same
   reason: that hand-over is the function's job. Two input vertices on one
   circle are one curve, and only one of them can start its one edge.
3. **Name a closed crossing by its two surfaces and which of their closed
   crossings it is.** This is the `which` that `CornerName::Meet` already
   carries for three surfaces (§31.3), applied to two. A plane through a
   ring's axis is the case run here. `intersect` gives it one run per circle,
   each picked by its own half-space, and which one keeps a vertex is one
   exact sign. `holds.rs`'s `the_half_space_naming_the_meridian_circle_through`
   already asks it, to tell a cap's two disks apart (§250.5). This is the
   family to file. It has to reach every name keyed by the two surfaces
   alone: `on_curve` and `taken_over` in `with_each_closed_curve_started_once`,
   `CornerName::RoundACurve` and `HalfWayRoundACurve`, and the second field
   of `EdgeName`, which comes from `which_curve_it_is`.

Other pairs that cross in two closed curves were **not run**: a plane square
to a ring's axis, which cuts it in an inner and an outer circle; two rings;
and a plane through a ring's axis cutting a whole ring that has no vertex on
either circle, where both circles would be started at `RoundACurve` of one
name.

### 378.5 What is left

- **Q512** is already the row for this refusal. It was filed from §357.4 and
  asks for a trace first. This section is that trace, so Q512 can become the
  family 3 fix, with `crates/rcad-topo/src/fuse/assemble.rs` in its write
  set. The test it turns round is
  `two_quarter_bends_meeting_end_to_end_are_refused_as_not_a_manifold_when_united`.
  Once fixed, it should check the union validates and measures `4π²` inside
  its bound, both ways round.
- **Q513**, the same pair with the second bend's axis turned round, waits on
  Q512 and is not touched here.
- The three pairs at the end of 378.4 are guesses that nobody has run.

### 378.6 The fix: family 3, a closed crossing named by which circle it is

*Queue row Q756, which replaces Q512, off `51a2d890`. It changed
`crates/rcad-topo/src/fuse/assemble.rs` and the two test files named below.*

**On the base**, `two_quarter_bends_meeting_end_to_end_are_refused_as_not_a_manifold_when_united`
with a print of what came back: both orders gave
`Err(ResultIsNotManifold { count: 4 })`, "the result would not be a solid: an
edge would have 4 faces along it". Q656's union of names (§284.8) did not
change it.

**The fix.** `name_of_the_closed_crossing_through` names a closed crossing by
its two surfaces, as before, and by `which_meridian_circle_it_is`: for a plane
and a ring, the four numbers of the half-space
`the_half_space_naming_the_meridian_circle_through` in `holds.rs` picks the
circle through the side's own point by. Every other pair gets `None` and is
named as before. The name reaches all four places 378.4 listed: `on_curve` and
`taken_over` in `Corners::with_each_closed_curve_started_once`,
`CornerName::RoundACurve` and `HalfWayRoundACurve`, and `EdgeName`'s
`which_curve_it_is`. The fuse's policy and log now travel down to them, since
the half-space is found by `intersect`.

**Why the four numbers are one name per circle.** `intersect` writes the two
circles' half-spaces as `h` and `h` turned round, with `h`'s normal the ring's
axis crossed with the plane's normal. Turning the plane or the axis round
negates every number of both exactly, so the half-space that keeps a given
circle holds the same numbers in every spelling, except that a zero can come
back `-0.0`. Adding zero turns it back. The test
`each_circle_a_plane_through_a_rings_axis_cuts_is_one_name_however_the_two_are_spelled`
in `assemble.rs` asks all four spellings: with the fold taken out, the four
come back as four names differing only in a zero's sign, and the union below
fails too. Turning the two surfaces the one way before asking was tried and
dropped: taking it out failed no test, because the fold already does the work.

**The answer now.** The test is renamed
`two_quarter_bends_meeting_end_to_end_unite_into_a_half_bend`, and checks both
orders: one solid, an empty log, `Model::validate` passing, the two caps on
`y = 0` facing `−y` and the two walls, and the volume inside the bound
`measure` gives at deflection `0.01`. A probe that printed the volumes gave
`39.41226215940417 ± 0.7842662817045987` and `39.41226215940403 ±
0.7842662817045987`, against `4π² = 39.47841760435743`.
`two_quarter_bends_on_one_ring_turned_round_get_past_the_crossing_either_way_round`
in `two_frustums_stacked_on_one_cone.rs` said the one-spelling pair refused
`count: 4`; its doc now names this answer as the one to compare against.

**§377's ten refusals answer too.** The same probe, run on §377's helper
`a_bend_turned_and_written_out`, fused the half bend with each quarter under
every rule §377.4 pins as `NotManifold`. All ten come back one solid with an
empty log that validates: the four unions and the two differences that should
be the half bend measure `39.412… ± 0.7843`, two quarters, and the four unions
that should be three quarters measure `59.118… ± 1.1764` against
`6π² = 59.2176`. So `tests/a_half_bend_fused_with_a_quarter_bend_on_one_torus.rs`
failed on the first run's branch, in its two tests that pin `NotManifold`, and
nothing else in `rcad-topo` did. That file was not in the first run's write
set, and the branch could not merge.

**The second run moved those pins**
(`lane-q756-two-quarter-bends-end-to-end-on-one-ring-2`, which merges the first
run's `a5cd7d68` onto `c11bf21f` and takes Q788's write set with it). The ten
are pinned to what they answer, each checked to validate and to measure inside
its bound; §377.4 and §377.5 have the table and the volumes.

### 378.7 What this leaves

- **§377's pins**, done in the second run. In
  `a_quarter_bend_inside_a_half_bend_fuses_to_the_quarters_it_should` the two
  unions are now `Quarters(2)`, and in
  `a_quarter_bend_against_a_half_bends_cap_fuses_to_the_quarters_it_should`
  the difference with the half first is `Quarters(2)` and the two unions
  `Quarters(3)`. §377.4 and §377.5 moved with them, and Q788, filed to trace
  those ten, is answered by this fix.
- **Q513**, the pair with the second bend's axis turned round, still refuses
  `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, before any of this is reached.
- **Other pairs that cross in more than one closed curve** are still named by
  their two surfaces alone: a plane square to a ring's axis, which cuts an
  inner and an outer circle, and two rings. Nobody has fused either.
  `holds.rs`'s `the_picker_naming_the_circle_through` already picks the inner
  and the outer circle by a pipe, and is private.

### 378.8 The three pairs 378.4 did not run, run

*Queue row Q757, off `d0eb84cc`. It changed no source: it adds
`crates/rcad-topo/tests/a_ring_crossed_in_two_closed_curves.rs` and this
subsection.*

**The pairs.** Each crosses a whole ring in two closed curves that no edge or
corner of either solid lies on, so both would start at a corner the two
surfaces alone name. A whole ring is two faces on the one torus, each closed
edge a rim naming the plane that cut it (`EdgeRun::RightRoundWhereItCrosses`);
all three rings validate and measure `8π²` or `10π²` inside their bound.

1. The ring of four round a tube of one, cut along its equators, and the block
   `[−6, 6] × [−6, 6] × [−2, ½]`, whose top cuts it in circles of radius
   `4 ± √3/2` at `z = ½`.
2. That ring and the ring of five cut the same way, both about `+z`, crossing in
   circles of radius `4½` at `z = ±√3/2`.
3. The ring of four cut along the two circles `y = 0` cuts it in, and the block
   `[−6, 0] × [−6, 6] × [−2, 2]`, whose side `x = 0` cuts it in the circles round
   `(0, ±4, 0)`.

The test file's doc has each rule's closed form by Pappus.

**What each gives**, `mise exec -- just t -p rcad-topo --test
a_ring_crossed_in_two_closed_curves`, five tests passing: all eighteen fuses
are refused, and each model is sound afterwards. Pairs 1 and 3 are refused
`Geometry(ACrossingBetweenAPlaneAndATorusThatIsAQuartic)` under every rule both
ways round, and pair 2 `Geometry(ACrossingBetweenATorusAndAnotherCurvedSurface)`.
So none of them reaches the naming this section is about.

**Where pairs 1 and 3 are refused.** `cuts::gather` asks `intersect` of every
pair of faces of two solids it cannot prove apart, through `cuts_between` and
`how_two_faces_meet`. `where_a_plane_and_a_torus_cross` in `rcad-geom`'s
`intersect.rs` refuses every plane parallel to the axis and off it, at its
`holds_the_centre` question, though the block's sides are 6 from the axis
against the ring's reach of 5 and cross nothing.
`only_the_sides_of_each_block_that_miss_the_ring_are_refused_against_it` asks
`intersect` of each block face against the ring: the face that cuts it gives
two curves, the faces square to the axis beyond the tube give nothing, and
only the sides are refused. **A gap**, and **Q618** is already its row: the
answer is nothing, one sign question away.

**Past Q618, in a scratch copy only.** With that question answered nothing for
a plane further off than `1.01 (R + r)`, in a copy of the base exported with
`git archive`, the same eighteen fuses gave:

| pair | union | intersection | ring less block | block less ring |
|---|---|---|---|---|
| 1 | `ACurvedFaceGoesRightRoundWithNoSeam`, both ways | panic in `rings_round`, both ways | panic in `rings_round` | `ACurvedFaceGoesRightRoundWithNoSeam` |
| 3 | panic in `holds.rs`, both ways | the same | the same | the same |
| 2 | unchanged | | | |

The `rings_round` panic is `assemble.rs:1448`, "every face joined here brought
an outline of its own", from `rings_of_each_joined_face` and
`join_up_what_is_one_face` under `assemble`. Whether it is the inner and the
outer circle sharing one name, as 378.3's two circles did, was not run. The
refusal was not traced.

**The `holds.rs` panic is on the base, without any fuse.**
`SolidView::classify_point` asked of a point inside the tube, `(0, 4, 0)` or
`(4, 0, ½)`, panics for both whole rings at `holds.rs:830`, in
`what_a_ring_face_between_two_planes_holds` under `what_a_curved_face_holds`,
`what_this_face_holds` and `classify::count_where_the_ray_meets_it`: "a face
this was sent is trimmed by planes only, and this one by Torus". The rim's
`and` is a plane, so `is_a_ring_face_between_two_planes` sends the face there,
and the face across the rim is the other half of the ring. Points outside the
ring, `(0, 0, 0)` and `(10, 1, 0)`, answer `Outside`. The test file does not pin
it: a panic is not a result.

**Where pair 2 is refused.** `intersect` itself, `intersect.rs:223`, for every
pair of a torus and another curved surface that is not the same ring. **A
gap**: `ACrossingBetweenATorusAndAnotherCurvedSurface`'s own doc in
`rcad-geom`'s `error.rs` names the coaxial pairs as two circles each and one
sign question apiece, and this pair is one of them.

**What this leaves.** Q618's write set does not hold the new test file, whose
pairs 1 and 3 its fix moves onto the two panics above. The `holds.rs` panic,
the `rings_round` panic and the refusal past Q618, and the coaxial rings, are
each a row of their own. The naming 378.4 asked about is still not reached by
any of the three.

### 378.9 Q1376: a bend and a bar on its axis get through intersect, the corner and the arrangement; walking the crossing is what is left

*Queue row Q1376, two lanes off `667c4a46`: `9b192c2d` and `90831e2d` on
`lane-q1376-a-bend-and-a-bar-on-its-axis-answer`, merged here as `90831e2d`
onto `lane-q1376-a-pipe-on-a-bends-axis-names-two-circles`.*

**Naming.** `9eead79c`'s naming — a ring and a ball, a pipe or a cone on its
own axis is the circles the two share, each named by the plane square to the
axis it sits in, up to two for a pipe or a ball and up to four for a cone — is
back, this time as one function,
`the_heights_a_ring_crosses_a_round_surface_at`, read by both `intersect` and
`three_surfaces`. It is `the_heights_a_ring_crosses_a_round_surface_at`'s doc
comment, not a holdback this section used to cite, that is current now.

**The corner.** `three_surfaces`'s `what_a_plane_through_a_rings_axis_leaves`
takes a sixth road through `where_one_plane_and_two_curved_surfaces_meet`: a
plane holding the ring's axis cuts each shared circle in two points, one
either side of the axis, read off the other surface's reach along that axis.
For the bend of ring four and tube one met by a bar of radius four on its
axis, that is `(±4, 0, ±1)` exactly — the corner
`what_a_plane_through_their_one_axis_leaves` already gives a bar and a cone
on an axis, now given a ring on it too.

**The arrangement.** `rcad-topo`'s `fuse::arrange` now works out the picker of
every run of a pair once, in `crossing_for`, and keeps it on the `Crossing`: a
plane-named circle is picked by the planes halfway to its neighbours, which
`intersect` proves part them, and it is ordered as the round surface and the
circle's plane, by the way their gradients cross where the circle starts.
Every face of the bend and the bar now splits where the ring's axis holds the
bar — `pickers_of` no longer panics on `RightRoundWhereItCrosses` for this
pair, and ordering it no longer refuses `ACrossingThisCannotBeOrderedAlong`.

**What still refuses, and why it is a gap and not this section's naming
again.** All twelve fuses the two bends give — two bends (ring four tube one,
ring six tube two), three rules, both ways round — still refuse
`ACrossingBetweenATorusAndAnotherCurvedSurface`, traced with a backtrace probe
on all twelve. Not from `intersect`, from `three_surfaces`, or from
`fuse::arrange` any more: from `rcad-geom`'s `walk.rs`,
`what_a_run_between_these_two_lies_on`, reached by `assemble`'s
`nest_the_shells` measuring the fused solid's volume through
`how_much_space_a_curved_solid_encloses` and `mesh`. The answer's arc of the
circle the ring shares with the bar is named `PartOfWhereItCrosses` between
the ring and the bar, with a chord half-space telling one part from the other,
and `what_a_run_between_these_two_lies_on` has no row that matches a ring
against a curved surface — every row it has is two of a plane, a pipe, a cone
or a ball. The circle exists, `intersect` has named it and the arrangement has
split on it; the arithmetic to walk it is the one piece not written. That is a
**gap** (§16.10), not a limit. It is named here without a queue id: the row
for it is filed when this branch lands, by `just queue apply`, not by this
file.

**The same gap reaches a file-stated pair, not just this section's bar.**
`crates/rcad-geom/tests/the_degeneracies_a_crossing_has_to_answer.rs`'s pipe-
into-a-bend test, built from the shapes one corpus KiCad file panicked the
evaluator on, crosses the same way now that `intersect` answers it: the
pipe's radius is the ring's radius plus its tube's radius to the digit, so in
the reals the pipe rests on the tube's rim in one circle, but `0.5 − 0.4`
stored as `f64` rounds one bit short of `0.1`, so the room `intersect` finds
is a tiny positive number instead of zero and it names two circles,
`2.356e-9` apart, rather than one. Walking either one still refuses under this
section's name, for the same reason: no row in `walk.rs` for a ring against a
curved surface, coaxial or not.

### 378.10 Q1396: walk answers; the twelve fuses now refuse a meshing gap instead

Row Q1396 (`bc6eb2d4`) gave `walk.rs`'s `what_a_run_between_these_two_lies_on`
the row §378.9 found missing: `where_a_ring_and_a_surface_on_its_axis_cut`
names the arc `assemble` built — the ring and the bar, between two corners,
round the circle the bar and that circle's plane cross in — by picking the
plane `intersect` already named it by and walking it as the bar and that
plane. The bend and the bar on its axis no longer refuse
`ACrossingBetweenATorusAndAnotherCurvedSurface` at all.

**What refuses now instead.** All twelve fuses (two bends — ring four tube
one, ring six tube two — three rules, both ways round) refuse
`ACurvedFaceGoesRightRoundWithNoSeam`, raised from `rcad-topo`'s `mesh.rs`,
`BuildingAMesh::cut_a_cap_up_to_its_pole`, while `assemble`'s `nest_the_shells`
meshes the answer to measure it
(`a_bend_and_a_bar_through_its_tube_on_its_axis_are_refused_where_a_piece_of_the_bend_is_meshed`
in `tests/how_often_a_fuse_with_a_torus_refuses.rs`). On ring four tube one the
refused wire, in the torus's chart of (round the tube, round the ring), has
corners `(π/2, 0)`, `(3π/2, 0)`, `(3π/2, π/4)`, `(3π/2, π/2)`, `(5π/2, π/2)`,
`(5π/2, π/4)`: it takes the start cap's inner half and the end cap's outer
half, so it climbs a whole turn round the tube instead of the half each cap's
own half-space bounds. A **gap** (§16.10), not this section's naming again —
Pappus gives every answer, the shared part being a quarter of
`π²Rr² − 4πr³/3`.

**Open as Q1402** (tier 1): the piece is built from the wrong pair of halves,
and a lead is named there but not run — not traced here.

### 378.11 Q1402 closed: all twelve now answer, within error of Pappus

Row Q1387 (merged `0315d17e`, second lane `d5a2cbc7`) gave
`which_arc_of_a_circle_this_is` in `assemble.rs` a plane-and-ring arm: a
section naming the two circles a plane cuts a ring in, and why the sign rule
holds on each. All twelve fuses §378.10 found refusing
`ACurvedFaceGoesRightRoundWithNoSeam` now answer instead, validated and
within error of Pappus's shared part. The old pin,
`a_bend_and_a_bar_through_its_tube_on_its_axis_are_refused_where_a_piece_of_the_bend_is_meshed`,
is gone; the new one,
`a_quarter_bend_and_the_bar_of_its_own_radius_fuse_by_every_rule_to_what_pappus_gives`
in `tests/a_bend_under_two_planes.rs`, pins the twelve answers. Q1402 is
closed, `done:d5a2cbc7`.
