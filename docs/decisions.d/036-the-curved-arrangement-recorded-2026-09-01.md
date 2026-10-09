## 36. The curved arrangement, recorded 2026-09-01

Lane 4 of §31.6: `fuse/bake.rs`, `fuse/cuts.rs` and `fuse/arrange.rs`, splitting
curved faces at their crossings, built under `Error::FuseNeedsFlatFaces` and
leaving it on for lane 5.

The headline is one sentence and it is §31.7 criterion 12: **a block with a bar
through it comes apart into a square with a round hole in it, the disk that came
out, and a wall in three rings**, and the same pair turned about an awkward axis
does the same with ellipses everywhere and nothing approximated.

§31.10's prediction was that M4's money goes to `fuse/arrange.rs`, *and not the
parts about lines*. **It is right, and §36.2 scores it line by line.** The
straight-line ideas were not where the names said they were.

### 36.1 What a face is now, and the two things §31.3 got wrong

§17.4's "everything that matters to a face is a pair of planes" is now the same
sentence with one word changed, all the way through: `BakedFace` carries a
`Surface`, `BakedFace::cuts` is a list of `Surface`, `BakedEdge::across` is the
surface of the face on the other side, `Piece` carries a `Surface`, and
`FaceMeet::carried_by` is a pair of them. `cuts.rs` no longer asks
`Plane::agreement_with` and then `planes_are_parallel`; it asks `intersect`,
which is the one authority on how two surfaces meet (§24.3) and settles the same
three cases in one call. Two faces on one **cylinder** are the coincident case
for free, which is what §31.3 predicted and it cost nothing.

**§31.3 is wrong about the seam, and the demonstration is `seams_are_left_out`.**
That section says a seam is an ordinary boundary edge whose second surface is
read out of its own `EdgeRun::PartOfWhereItCrosses::and`. It cannot be one: a
seam is used **twice by one face**, so both uses run along one arc of one
crossing in opposite directions, and an arrangement that took it as boundary
would have one stretch of one curve claimed by two sides of one wire at once — a
winding count reads that as `+1` and `−1` about the same region. What a seam is,
is where the surface's own two coordinates were cut open and not where the face
stops, which is the sentence `holds.rs` already says on the other side of the
crate. So it is left out, and a bar's wall comes back bounded by its two cap
circles and a ball's skin by nothing at all.

**§31.3 is also wrong that a corner has two routes and not three.** It settles
"three or more surfaces round it is re-derived, two is read out of the model".
A **pole of a ball** has *one* face round it and one surface, and the old
`planes_around_a_vertex` asserted at least three. What landed is "three or more
is re-derived, fewer is read and moved", with a pole handing the same surface
over twice — which is honest rather than a fudge, because the box round a moved
corner is derived over two chains and a pole's two chains are the same one.

One correction inside that: **a solid that has not been moved is not moved**.
`where_a_corner_lands_after_moving` publishes a worst-case reach and applies it
whatever the movement is, so a bar standing where it was built would have come
back as a box — and §17.7's whole lattice control is that a lattice model decides
exactly. Baking checks for the identity first.

### 36.2 §31.10's prediction, scored line by line

The prediction names three functions and says the vocabulary of `Line::axis`,
`forward_climbs_the_axis` and `first_arc_met` is "a straight-line idea sitting
inside a function whose name says nothing about it". Every one of those is a
**real** assumption. So are three the prediction does not name, and they are the
expensive ones. And two things that look like straight-line ideas turned out to
be nothing of the kind.

| What it was | Real assumption, or only a name | What it cost |
|---|---|---|
| `Line::axis`, `forward_climbs_the_axis` | **real** | `Crossing::order` is now `AlongAnAxis` or `RoundAClosedCurve`, and a closed one is ordered by `which_comes_first_round_a_crossing` from the place §34.4 made a function of the pair |
| `nodes_along`, `order_along` | **real**, and they are the axis rule again | a cyclic order is not a total one until somewhere is called first, so a closed crossing sorts by insertion against its own start |
| `first_arc_met`, `piece_around` | **real** | a ray needs a direction to fire along and a bounded face to hit, and a closed curve has neither. Kept for lumps that hold an endless curve, and a second route added beside it |
| `drop_the_lumps_that_are_only_cuts` | **real, and the name says nothing at all about it** | its argument is *"a cut line that runs through the face crosses its boundary to get there"*, which is true of a line and false of a circle. The circle a bar cuts in a block's face lies wholly inside it and touches no boundary, and dropping it lost the hole. Now "lumps of **straight** cuts alone" |
| `pieces`, `the_cycle_of_the_lump_that_is_wound_once`, `outer_lump` | **real, and the deepest of them** | a piece was one wire and its holes, read off the lump the face's first wire is in. A bar's wall between two cap circles has two boundary curves in two lumps and neither is outer (§31.3). A piece is now a **cell of the surface**: a group of walks, joined across lumps by which side of every closed curve each lies on |
| `cut_the_lines_into_arcs` | **real, in one word** | a closed crossing wraps, so `n` nodes leave `n` arcs and not `n − 1`, and one node on its own leaves one arc that starts and ends there. That one word is what a cap circle and a round hole are made of |
| `turns_anticlockwise` | **only a name**, exactly as §31.2 row 9 predicted | `orient3d` of three stored normals became `orient3d` of three gradients, and for three planes the gradients *are* the stored normals, so the arithmetic is bit for bit what it was |
| the cycle walk — `walk_the_cycles`, `next_around_the_cell`, `sorted_anticlockwise`, `half_of_the_turn` | **only a name**, and it is the largest single piece of the file | not one line changed for curvature. Keeping the region on the left and taking the next half-edge clockwise from the way back is a statement about a graph drawn on a surface and never about which surface |
| `count_the_windings`, `spread_the_winding` | **only a name** | crossing a boundary arc from the right of its wire to the left adds one, on a sphere as on a plane. What changed is what it counts *over* — cells instead of walks — and that is the row above, not this one |
| `relation_of_two_lines` | **only a name, and it is still the right answer** | §31.2 row 7: all planes and it is four exact minors, otherwise it is "are the two partners one surface" and `intersect` answers. Both are there and the planar path is untouched |

**So the money went where §31.10 said and not to what it named.** The two
functions it calls out by name — `piece_around` and `first_arc_met` — are still
in the file and still work; what had to be rewritten around them is what decides
*which walks bound one region*, which the prediction does not mention and which
is a third of `arrange.rs`.

### 36.3 Two walks that disagree about a closed curve bound different cells

*Corrected by §340. This heading said "exactly when they agree about every
closed curve", and the paragraph below said it again. Agreeing is not enough:
§282 found a frustum's face where it glued two cells into one, and queue row
Q20 found the same glue making a fuse panic. §340 says what is asked as well.*

This is the new idea and it is worth stating on its own, because it is what
replaces the ray for everything a ray cannot reach.

A curve drawn on a face is **drawn in full** when it closes and none of its arcs
was thrown away. Such a curve cuts the surface into exactly two, so *which side
of it* is a sign — read off the half-edge where a walk uses it, and off any of
the walk's own nodes where it does not. Two walks in different lumps of the
drawing that disagree about a curve drawn in full bound different cells. Two
that agree about every one may still not share a cell, when a curve that runs
on for ever is what tells them apart (§340.1).

A curve that runs on for ever is never drawn in full, and that is the load-
bearing word rather than a caveat: the stretch of a line past its outermost node
is not drawn, so the sign of it says the same thing on both sides of a face's own
rim and cannot tell the inside from the outside. Two things stand in for it. A
lump holding an endless curve is placed by `piece_around`'s ray, unchanged. And a
lump of closed cuts on a **flat** face is placed by counting how often the face's
own rim winds round one of its nodes.

Both of the ways that count can fail turned out to matter, and both are the same
shape as §35.2's regression — *a fact about the drawing that arithmetic cannot
see*:

- **A fan cannot answer about a point on one of its own chords, and every chord
  of a square runs through its centre.** `select.rs`'s `winding_around` fans from
  each corner in turn, and for a rectangle the chords are the two diagonals, so
  no apex helps. The circle a bar cuts in the middle of a block's face starts at
  a point that is often exactly there. What stands in its place is a crossing
  count along a half-line, which is `holds.rs`'s route over boxes — **a third
  spelling of one question**, and §36.7 hands the unification on.
- **The corner a half-line is aimed at is on it by construction**, and saying so
  is not an optimisation. The determinant that would work it out has the same box
  in two of its rows, and an interval evaluation of one like that straddles zero
  however far away everything is. Every corner of a placed solid is such a box,
  so before this was written down the count refused on every turned pair.

### 36.4 Four families, and none of them refuses

Family by family and never blended (§17.7). 256 drawn pairs per family, every
face of both solids cut, `Policy::certified()`.

| Family | faces cut | refused | what they named |
|---|---|---|---|
| lattice | 2304 | **0.00%** | — |
| turned | 2304 | **0.00%** | — |
| leaning | 2304 | **0.00%** | — |
| written at `2^k` | 2304 | **0.00%** | — |

**A zero that is luck is a zero that moves, so here is why this one does not.**
The turned family started at **22.09%** and every one of those refusals was about
a curve that reaches no part of the face it was hung on. A block wall a hair off
parallel to a bar's axis cuts that bar's cylinder in an ellipse **ten
quadrillion** across — the measured node was at `−4.2e16` — and three points on
one are so nearly in a line that ordering them is an honest refusal. The same
wall *exactly* parallel to it cuts two straight lines whose direction has to be
read off a gradient at a box that far out, and that is the other half:
`which way a crossing runs` at 235 of 2304, `order along a crossing` at 274.

A **cut** all of whose nodes are provably clear of the box round the face cuts
nothing off it, and is left undrawn. It is a proof and not a shortcut: a curve
that crossed the face's boundary would share a node with one of the crossings the
boundary runs along, and every meet of every pair has been worked out by then.

**The word "cut" in that sentence is load-bearing and was measured to be.**
Written as "a crossing", it drops curves the face's own rim runs along — and the
rim of a face is not optional however far its nodes are. Eight tests of the
planar suite said so, and they are the demonstration.

Two families were re-measured rather than drawn fresh, because the generator's
first version drew knife edges: a bar exactly tangent to a block's wall is
roadmap §7.4's own catalogue entry and not what a *control* is for (§17.7). The
bar is kept a whole unit clear, and the entry stays in the catalogue where it
belongs.

The intersector's own rates are untouched by this lane, which is what they should
be — `rcad-geom` has **0 files changed** — and so are §35.8's classification
rates.

### 36.5 `Picker` did not come due, and that is a finding rather than a reprieve

§33.8 and §34.9 both hand lane 4 the same trap: *"`Picker` has no `transformed`,
and this comes due at you, when a fused solid holds edges from two placed
arguments."* **It did not, and the reason generalises.**

Nothing in the arrangement ever moves a picker. Where two surfaces cross in more
than one curve, the arrangement asks `intersect` about the two surfaces *already
placed* and is handed the pickers of the moved crossing — so the only stored
picker a fuse could have read is the one saying which half of a meridian a seam
is, and §36.1 leaves seams out. The rule to carry forward is one sentence:
**ask the intersector about the surfaces where they are, rather than moving what
it said about them where they were.**

That leaves §33.8's warning intact rather than discharged. A `HalfSpace` still has
no canonical form, still must never be compared or deduped or keyed on, and
`Picker::side_of_point`'s `Undecidable` still rules nothing out — which is why a
boundary edge whose corner is held by two curves of one pair, or by none, is
`CannotTellWhichCurveOfACrossingAFaceRunsAlong` and never a guess.

### 36.6 What each criterion in this lane's reach cost

| | | |
|---|---|---|
| **12** | the pieces a curved face is cut into cover it and overlap nowhere | met. `the_pieces_a_curved_face_is_cut_into_cover_it_and_overlap_nowhere`, over every face of a block with a bar through it. The oracle needs no area and so reads the same on a curved face as on a flat one: every side is walked once, a side two pieces share is walked once each way, and a side nobody walks back along is on the rim |
| **16** | a lattice curved control at 0% | met for this lane, and the other three families are at 0% too. §36.4 |
| **11** | `Error::FuseNeedsFlatFaces` gone | **not this lane's**, and it stands. Everything under it is built |
| **13** | a fused curved solid meshes | not this lane's; nothing is committed yet |
| §30.8 item 10 | a closed edge that is a hole in a curved face has no direction rule | **closed**, and it cost one line of algebra rather than a predicate. `(∇S × ∇T)` crossed with `∇S` and dotted with `∇T` is `−‖∇S × ∇T‖²` by Cauchy–Schwarz, so **left of forward is always the negative side of the partner**, everywhere, with no case analysis — and a wire walks a closed edge forward exactly when the face's material is inside the partner. `a_bar_wall_is_bounded_by_two_cap_circles_that_run_opposite_ways` |
| §30.8 item 11 | is `Undecidable` out of `mesh` reachable | **still not**, and §31.8 keeps its lead. Nothing here changes what a committed vertex is |

### 36.7 What lane 5 has to be told

Written for a stranger, because the same head reading it a day later is one.

- **The gate is still on.** `Model::fuse` refuses a curved solid at
  `fuse/mod.rs`, and `Piece::plane()`, `select.rs`'s `outward_plane_of_face` and
  `assemble.rs`'s whole `Face` are still written in `Plane` behind it. Each of
  those `expect`s says so in its own doc. Taking the gate off is taking those
  three with it.
- **A piece may have no outer wire, and `Piece::wires` says so.** On a flat piece
  the first is outer and the rest are holes, which is what `Model::validate`
  reads. On a closed surface the order is the order the walks were found in and
  nothing claims more (§31.3). `check_curved_face` has no winding check and
  §35.8 records that half of criterion 15 as open; this lane did not close it.
- **The seam is gone from every piece and has to be put back.** A piece of a
  bar's wall that goes right round the cylinder needs one to be meshable, and
  `mesh.rs` is where that bites. Nothing in the arrangement can invent it,
  because a seam is a *choice* of where to cut the surface open, and the commit
  is the first place that choice belongs.
- **Three refusals are the shape of what is left, and all three are gaps.**
  `ACrossingThisCannotBeOrderedAlong` for a quartic — a ball with a pipe through
  it, and §34.9 named this lane as where it arrives. `CannotTellTheCellsOfAFaceApart`
  for two regions the sign rule joined that the surface keeps apart.
  `CannotTellWhichCurveOfACrossingAFaceRunsAlong` for a corner two curves of one
  pair both hold. §36.8 classifies every one.
- **A quartic closes, and finding a place to start on one is not being able to
  order it.** Reading those two as the same question gave a ball with a pipe
  through it **four** pieces where it has three, in silence, until they were told
  apart. Whatever else changes, `how_a_crossing_runs` must keep dispatching on
  the *pair* and not on whether a start could be found.
- **There are now three spellings of "how often does this wire wind round this
  point".** `holds.rs`'s crossing count over model corners, `select.rs`'s fan over
  the arrangement's boxes, and `arrange.rs`'s crossing count over the same boxes.
  §35.10's last trap asked for one and there are three. The fan is the one that
  cannot answer at a rectangle's centre; the two counts are the same algorithm
  over two point types. Whoever unifies them should keep the count.
- **`Bounds::provably_misses` is a proof and the box behind it has to stay one.**
  `SolidView::box_around_one_face` bounds a curved face by an argument per surface
  kind, and a flat face by its corners *plus the box of every face across a side
  of it that is not straight*. Tighten either and the drop in §36.4 stops being
  sound.
- **`stretches_of_a_line_inside` says nothing about a curved pair.** It measures
  stretches along an axis and a closed crossing has none, so it comes back empty
  and the self-crossing check `cuts.rs` runs is silent about a solid whose curved
  faces run through each other. A **gap**, and what closes it is a way to say
  where two arcs of one closed curve overlap.
- **The coplanar joiner is only asked about a wholly flat solid.** It is written
  in planes, and nothing in this milestone builds a curved solid with two faces on
  one surface. A curved *answer* of this fuse could have one, and it arrives as
  `AFaceMeetsItsNeighbourOnItsOwnSurface` rather than as a wrong arrangement.
- **`rcad-predicates` is still frozen**, at 0 files changed for five lanes now,
  and `rcad-geom` is at 0 for this one.

### 36.8 Every refusal this lane can produce, classified

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `ACrossingThisCannotBeOrderedAlong` | the two surfaces cross in a quartic — a ball against a pipe, or two pipes | **gap** | `a_ball_with_a_pipe_through_it_is_refused_rather_than_cut_wrongly`. The curve is there and closes; what is missing is an order along it, which takes the marcher rather than a determinant. §34.10 files the same one dimension down. **2026-10-09, Q1483:** not for two pipes where the fat one swallows the thin one whole: their two loops each run once round the thin pipe's axis, and a determinant about that axis orders them (§421) |
| `CannotTellTheCellsOfAFaceApart` | two regions of one face were joined that the surface keeps apart | **gap** | telling them apart takes a point inside each, which is a construction this arrangement does not make. Not a broken invariant, because a face that really did overlap itself looks the same from here and only one of the two is a bug in this crate |
| `CannotTellWhichCurveOfACrossingAFaceRunsAlong` | a boundary edge's corner is held by two of the curves its pair names, or by none | **gap** | a picker's `Undecidable` rules nothing out (§33.8), so both come back held. What closes it is a picker that decides, which is §33.5's own four rows |
| `AFaceMeetsItsNeighbourOnItsOwnSurface` | two faces on one surface with an edge between them | **gap** | they carry no curve for that edge to run along. The joiner takes such a pair apart for a flat solid and is not asked about a curved one; what closes it is a joiner written in surfaces |
| `AFaceMeetsItsNeighbourNowhere` | a face has an edge along a curve its two surfaces do not have | **not a refusal at all** — a model whose topology and geometry disagree |
| `TwoCurvesTouchWithoutCrossing` | two curves on one face are tangent at a node | **gap** | there is no anticlockwise order round such a node and so no way round the cell it bounds. Following one of the curves past the touch closes it |
| `Undecidable` naming `where three surfaces meet` | no triple of surfaces round a corner could be shown to meet at a point | **gap** | inherited from `meet_of_three_surfaces`, and §34.10 classifies it there |
| `Undecidable` naming `order along a crossing` | two points on one crossing could not be told apart or put in order | **limit** where the two are one point and a **gap** where the arithmetic ran out; §36.4's four families reach neither |
| `Undecidable` naming `which way a crossing runs` | the direction of a crossing worked out at a box straddles every axis | **gap** | it was the turned family's whole budget and is now unreached, because the curves that provoked it reach no face |
| `Undecidable` naming `which piece of a face a hole is in` | no ray through a lump of endless curves gave an answer | **gap**, unchanged from §17 |

### 36.9 Corrections to §31, §33 and §34

- **§31.3's seam.** A seam is *not* an ordinary boundary edge and its second
  surface is not read out of its own run. §36.1, with the demonstration.
- **§31.3's two routes for a corner.** There are three cases and not two: three or
  more surfaces round it, exactly two, and **one** — a pole of a ball, which the
  old code asserted could not happen.
- **§31.3's "a corner read out of the model and then moved is a box".** True of a
  corner that was moved. A solid standing where it was built is not moved, and
  giving it a box anyway costs §17.7's lattice control its exactness.
- **§31.10's prediction** is right about the file and wrong about the parts.
  §36.2 scores it: `turns_anticlockwise` and the whole cycle walk are only names,
  and what had to be rewritten is which walks bound one region, which the
  prediction does not mention.
- **§33.8's first trap and §34.9's last** say `Picker::transformed` comes due at
  lane 4. It does not, and §36.5 says why the reason generalises.
- **§34.9** says `Plane::meet_of_three` can be switched for
  `meet_of_three_surfaces` and nothing lost. True, and one thing gained that the
  trap does not mention: **three surfaces meet at two points and the old check
  kept the first.** `place_the_crossings` skipped a place if *any* node lay on
  both curves rather than if one lay *here*, which was right while three planes
  met once. A cap plane cutting a bar's own cap circle is the shape it shows on,
  and what it left behind was a line that entered a closed curve and never came
  out.
- **§35.10's second trap** — `bake.rs`'s box round a face has no guard — is
  closed, and the box now hangs off the solid because the neighbour across a
  curved side has to be found already placed. `SolidView::box_around_its_corners`
  still has no kernel caller and this lane did not delete it.

### 36.10 The numbers

| | at §35 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1046 | **1062** |
| of those, in `rcad-topo` | 312 | **324** |
| suite duration | 2.9 s | 2.9 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 746 | **775** |

`just default` is green, `cargo-deny` included.

`just fuzz 180` ran 5 274 327 executions in 181 seconds, found **29 new seeds**
and **no crash**. Coverage stayed at 2506 edges, which is what it should be:
`rcad-geom` has 0 files changed, so there is no new arithmetic for the corpus to
reach.

**One test had its number re-derived.** `how_often_a_certified_fuse_of_a_skew_pair_refuses`
asserts which predicate the skew family's refusals mostly name, and the doc
beside it records "30 naming it against 21 naming `side of plane`". The 30 has
not moved — it is the same question about the same straight crossings, under the
name `"order along a crossing"` rather than `"order along a line"`. The 21 beside
it came down to **12**, because the gathering step stopped asking
`Plane::agreement_with` and then `planes_are_parallel` about every pair of faces
and started asking `intersect`, which settles the same three cases in one call
and gives up in fewer places on the way. The assertion is on the leading cause
and the margin widened rather than narrowed.

### 36.11 The gate is off, and what that cost

Lane 5 of §31.6: `select.rs`, `assemble.rs`, `mod.rs` and `history.rs`, and the
five lines at the top of `Model::fuse` that §31.1 calls M4's one deliverable.
They are gone, `Error::FuseNeedsFlatFaces` is deleted, and **a block with a round
hole taken out of it is one solid that validates and meshes** under all three
rules, both ways round.

| | | |
|---|---|---|
| **11** | the gate gone, and a block with a round hole is one solid | met. `the_fuse_takes_a_curved_solid_now`, and `Error::FuseNeedsFlatFaces` has no variant left to assert on — what stands in its place is the operation working |
| **13** | a fused curved solid meshes and passes §23.4's oracle | met for the first of its two shapes. `a_block_with_a_round_hole_meshes_into_a_closed_surface`, through `the_mesh_closes_up`, for Union, Intersection and Difference — **and the volume of each**, which is the strongest thing there is to say about a curved answer and is what says the tube is not inside out. `a_ball_with_a_pipe_through_it` is refused by §36.9's first row and stays refused |
| **16** | the two rates, family by family | met, and stronger than asked. Four families, three rules, both ways round, **0.00% refused and nothing recorded** on all four — including the turned one. §36.14 |
| **12** | the pieces cover the face and overlap nowhere | met by lane 4 |

Three things a curved answer needed that a flat one never did.

**Which way a face walks a closed edge is not something its two ends can say**,
because they are one corner. It is read off which side of the partner the
material is on — §36.6's identity, already in the arrangement — and *turned round
for whichever of the two faces did not create the edge*, because "forward" is
`∇S × ∇T` for that face's own surface and the two faces name `S` and `T` the
other way round from each other. Without that, both coedges of every cap circle
came out forward.

**Which surface a face is on is geometry and which way round it is used is
topology, and only a plane welds them together** (§31.3's ON split, arriving in
the commit). A plane turned round is four negated numbers and the answer stores
it that way; a sphere has no reversed form, so a curved face that a rule turns
round carries the turn on its *shell* instead. The same split settles the ON
columns: for a flat pair `Plane::agreement_with` answers "one surface" and "same
way" together, and for a curved pair `intersect` answers the first and the two
faces' own orientations answer the second.

**A candidate point inside a curved piece has to be put back onto the surface**,
because a blend of corners on a sphere is not on it. That goes through the
surface's own two coordinates, which is `atan2` and is allowed for the reason §5
gives — *choosing* which point to look at decides nothing, and what follows
proves it is inside the piece with signs that never see a coordinate. What comes
back is a **box** [`Surface::ROUND_TRIP_ROUNDOFF`] wide, and the width is not
slop anybody accepted: a candidate a rounding off a curved face would be
classified confidently against a solid whose own face it is lying on, and `ON` is
the one column a ray was never what settled.

And a piece bounded by **one** closed curve has one corner, so every blend of its
corners is that corner and every one of them is on the boundary. Two families
were added for it: the middle of a round wire, which for a flat piece is where
the ball or pipe that cut it has its axis, and six points spread round a curved
surface from the middle of what corners there are.

### 36.12 The seam a fuse cannot invent, and where it went instead

A fuse's answer contains a **tube**: a ring of a bar's wall between two planes,
bounded by two curves that each go right round the cylinder and by nothing else.
It is the first face in this kernel with a hole in it that is not flat, and
§35.7 predicted lane 5 would make one.

**It has no seam and cannot be given one.** A seam is an edge between two corners
at a single longitude, and the two curves bounding a tube start wherever the pair
that cut each of them starts — which is `where_a_closed_crossing_starts`'s answer
for each pair separately and is not the same longitude twice. Inventing a corner
at a chosen longitude does not help either: the plane the seam would run in has
to hold both corners *provably*, and a plane built through two rounded points
holds neither.

So the seam went into the mesher, where it is a cut in the surface's own two
coordinates and not topology anybody has to hold. `cut_a_ring_across` cuts a ring
at four longitudes at once and hands back four patches with one loop each, which
is the shape everything below it already expected. **The rule §35.7 and §31.10
item 5 said lane 5 would falsify is falsified in exactly one place**: the
sentence *"nothing this kernel builds is such a face"* is gone, and what is still
refused is a face with a hole that is **not** a ring — one curve right round and
another that is not — because then there is nowhere to cut across to.

Three things that had to be right for the ring cut, each of them a fact about a
chain that goes right round rather than an arithmetic bound:

- **A corner's distance to a longitude is not measured modulo a whole turn.** A
  chain that goes right round has a first corner and a last a whole turn apart,
  and modulo a turn they are equally near everything — so which came back decided
  whether the next stretch went a quarter of the way round or three quarters.
- **The two chains are walked in longitude frames of their own**, a whole number
  of turns apart, and one is put into the other's frame before anything is
  measured. A patch that took them as they came stepped a whole turn crossing
  from one to the other, which in the surface's two coordinates is a boundary
  folded across itself.
- **Every stretch is put on the end unwrapped against what is already there**,
  which is what `arrive` does one level up and for the same reason.

**What is still refused, and it is the shape M4 does not reach:** a cap a plane
cuts off a **ball**. Its whole boundary is one circle of latitude, and its patch
is a rectangle whose top edge is the pole and whose two sides are one meridian
used twice — and the model says neither of those about the face. A ring has
somewhere to cut *to* and a cap does not.
`a_ball_cut_by_a_plane_is_refused_by_name_rather_than_meshed_wrongly` asserts it
of the whole family and not of one shape, because a refusal that only fires
sometimes is one somebody has to characterise (§14.7).

### 36.13 A confident wrong answer, and two corners that could not be exact

**The worst thing found in this milestone, and the fuse is what asked the
question in anger.** `classify_point` said a point outside a bar of radius two
was inside it. §4.1 ranks that below a crash.

`gap_enclosing(a, b)` is the step *from* `a` *to* `b`, and the two callers in
`classify.rs`'s `gradient_at` never turned it round — so a ball's and a pipe's
gradient pointed at the middle instead of away from it, and *which way a ray
crosses a curved face* came out backwards every time.

**It cancels for a solid whose faces are all curved**, which is why lane 3's
grids of 9826 points against a bar and a ball never showed it: reversing every
crossing flips the total, and `winding == 0` is still `winding == 0`. What shows
it is a ray that crosses **one curved face and one flat one** — a ray through a
bar's wall and out of its cap — and there the two contributions that should
cancel add instead. Nothing in the kernel fired such a ray until selection asked
where a piece of a block's face sits relative to a bar.

Two more, and both are corners a fuse builds that a builder never did:

- **A corner of a curved face is asked for a sign and never for a distance**
  (§35), so a *constructed* corner — which every corner a fuse solves for is —
  could not pass `check_curved_face` at all: it is a rounding off its own
  surface and the exact level says so. What it can be asked is whether the box
  it already carries straddles that surface, which is §17.2 read literally. An
  exactly placed corner is still asked exactly what it always was, because the
  box is only consulted after the exact point has failed.
- **Where a closed crossing starts is a walked point and not an exact one.** §34
  says so in as many words — *"it is a point and not a box, and that is not a
  promise it is exact"* — and reading it as exact made it a corner of a curved
  face that the sweep could certify was off that face's own surface. It is a box
  [`POINT_ALONG_A_CROSSING_ROUNDOFF`] wide, unless **both** surfaces certify the
  triple `Zero`, which on the lattice they do — and there the box would have
  turned every sign the arrangement asks about it into a refusal.

One thing changed shape rather than being fixed. `Assembly::commit` re-validates
what it wrote, but only in a debug build and only as a *panic*: it is there to
catch a bug in this crate, and a release build writes the answer out regardless.
A curved answer whose two faces along one side disagree which way they face is
exactly that, so **the skin is checked for balance before anything is written**
and comes back as a refusal. Refusing beats corrupting, and both beat crashing on
a caller's solid.

### 36.14 The numbers

Family by family and never blended (§17.7). 64 drawn pairs per family, three
rules, both ways round — 384 fuses each — under `Policy::certified()`.

| Family | fuses | refused | recorded anything |
|---|---|---|---|
| lattice, a bar through a block | 384 | **0.00%** | 0 |
| turned about an awkward axis | 384 | **0.00%** | 0 |
| written at `2^k` | 384 | **0.00%** | 0 |
| a ball resting on a block | 384 | **100%**, one named reason | — |

**The turned row is the one to read twice.** A turned curved pair refusing
nothing is stronger than §31.7 asked for, and it is not luck: lane 4 took the
arrangement's own turned rate to zero by dropping cuts that reach no part of the
face (§36.4), and everything above it is signs about boxes that either prove or
give up — and none of these draws reaches one that gives up. The families here
are deliberately clear of knife edges, which is what a *control* is for; the
catalogue entries live in `degenerate_fuses.rs` and are lane 6's.

**The ball row is a refusal and not a rate.** Every fuse of it refuses, for the
one reason §36.12 gives, and the test asserts both — that the reason is the only
one and that it fires every time.

The intersector's and the classifier's own rates are unchanged by this lane
except where §36.13's defect moved an *answer* rather than a rate.

| | at §36.10 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1062 | **1067** |
| of those, in `rcad-topo` | 324 | **329** |
| suite duration | 2.9 s | 2.9 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 775 | **793** |

`just default` is green, `cargo-deny` included. **`rcad-predicates` is still at 0
files changed for the whole milestone**, across six lanes.

`just fuzz 180` ran 5 242 652 executions in 181 seconds, found **18 new seeds**
and **no crash**. Coverage went from 2506 to 2507 edges, which is what it should
be: `rcad-geom` is at 0 files changed for lanes 4 and 5 together, so there is no
new arithmetic down there for the corpus to reach.

**One prediction settled.** §30.8 item 11 said M4 would make `Undecidable` out of
`mesh` reachable through a fuse's constructed corner; §31.8 predicted it would
not, because `write_into` stores `middle_of(point)` and a committed vertex has
always been an exact triple. **§31.8 is right.** `Assembly::Corner::point` is a
`Point`, `write_into` hands it to `add_vertex` unchanged, and the mesher reads
vertices and walked points that are ordinary `f64` triples — so a mesh domain
still holds no constructed corner after M4. The width goes on the vertex as an
`Approximation`, which is where §17.2 says it goes and where §36.13 now reads it
back from.

**§31.8 item 9 is still not reached.** `MOST_POINTS_A_MESH_WILL_HOLD` is four
million and the largest thing built here is a block with a round hole meshed at a
deflection of a hundredth. §35.9 already corrected the watch and nothing here
moves it.

**§30.8 item 6 did not come due.** A crossing that runs through itself needs two
equal pipes whose axes meet, or a ball reaching exactly across a pipe; the first
is refused by `intersect` and the second by the quartic ordering, both before
anything could reach the third `EdgeRun` variant. §31.9's deferral stands and its
consumer is still not this fuse.

**§35.6's two refusals hold with the gate off.** `select.rs`'s
`CannotFollowAFacesTrimming → Ok(None)` and `assemble.rs`'s abandon-a-corner are
both about a face of a *caller's* solid, and taking the gate off does not make
either of them a broken invariant — what changed is that they are now reachable,
by a fused curved solid handed back to the fuse. Neither fires in §36.14's four
families.

### 36.15 What lanes 6 and 7 have to be told

Both are test-only, in `rcad-topo/tests/`, and both build on this.

- **Three shapes work and two refuse, and the line between them is one
  sentence:** an answer whose every curved edge is a **whole closed curve** is
  built, and one with an *arc* of a curved crossing in it is refused. A block
  with a round hole through it, a bar cut square across by a block, and a blind
  hole are the first; a bar notched by a block and a ball cut by a plane are the
  second. Write the catalogue against that line rather than around it.
- **A ball cut by a plane refuses**, so roadmap §7.4's tangent-sphere entries
  reach `ACurvedFaceGoesRightRoundWithNoSeam` before they reach anything about
  tangency. That is a real answer for the table and not a hole in it, but it
  means the entry is measuring the seam and not the tangency.
- **The metamorphic suite's generators are flat** (`tests/pairs/mod.rs` builds
  profiles and extrusions), and a curved family needs a `Shape` arm of its own. A
  bar through a block, with the bar a whole unit clear of the block's walls, is
  the family §36.14 measured and the one that refuses nothing.
- **A turned curved pair refuses nothing at all**, so the properties that hand
  back `Ok(())` when there is no answer will really run over it. That makes the
  *comparison count* guard (`SOME_COMPARISONS_MUST_HAPPEN`) worth setting high
  for a curved family rather than low.
- **Two coincident curved faces are kept as two.** The joiner only ever joins two
  flat faces, because the walk that rebuilds a joined face's rings winds corners
  about a plane's normal and has no curved twin. A curved answer is therefore
  sound but cut into more faces than it has, and a property that counts faces
  will see it. What used to make joining urgent no longer does: §17.1 could not
  re-derive a corner with two surfaces round it and §31.3's second route reads
  one out of the model.
- **`Model::fuse` is no longer gated on anything**, so a test that asserted a
  refusal by name has nothing to assert. `the_fuse_takes_a_curved_solid_now`
  replaced the last one.

### 36.16 Every refusal this lane can produce, classified

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` | an edge of the answer runs along part of a curved crossing | **gap** | `a_bar_notched_by_a_block_is_refused_by_name`. Two corners on a circle leave two arcs with the same two ends, and what tells them apart is a half-space through the chord square to the crossing's own plane — which exists, and which cannot be *built* here, because both ends have to be provably on it once its offset is rounded and nothing rounds it in a direction it can prove |
| `ACurvedFaceGoesRightRoundWithNoSeam` | a curved face is bounded by one curve right round its surface and nothing else | **closed at §36.18 where the surface pinches**, and what is left of it is not a gap: a face bounded that way on a surface with *no* pole is half an endless tube and is not a face |
| `ACurvedFaceCannotBeCutSmallEnoughToMesh` | cutting a curved face is not making the pieces narrower | **gap, and no generator reaches it** | it was an `assert!` until this lane, on the grounds that no builder makes such a face; `validate` walks it and `Model` is `Deserialize`, so what reaches it is a caller's bytes |
| `Invalid::FacesDoNotAgreeWhichWayTheyFace` out of the fuse | the skin the answer would have does not balance | **not a refusal about arithmetic** — it is this crate saying it cannot assemble what it cut, before writing anything. A release build would otherwise have committed it |
| `Invalid::VolumeIsNotPositive` | a shell of the answer encloses nothing | unchanged from §17, and now summed over a **mesh** for a curved shell, because a bar's wall fans into two triangles on its seam and a round cap into none at all |
| `Undecidable` naming `a point inside a piece of a face` | no candidate could be proved inside a piece and placed | **gap**, unchanged from §17.4, and now with four families of candidate rather than two |
| everything lane 4 can produce | | | §36.8 |

### 36.17 Corrections to §31 and §35

- **§35.6** says `select.rs:420` and `assemble.rs:943` are refusals rather than
  panics because what reaches them is a face of a caller's solid. That reasoning
  was written against a gate that was still on; with the gate off it holds
  unchanged, and §36.14 records that neither fires in the measured families.
- **§35.7** says `CurvedFaceWithAHoleGoesTooFarRound`'s claim is untouched by a
  washer and that what falsifies it is a ball with a pipe through it. What
  falsifies it is a **tube**, which is what a block with a round hole leaves of
  the bar's wall — a simpler shape than the one predicted, and one this lane
  builds on its first try.
- **§31.10 item 5** watches two claims in `mesh.rs`. Both are now false as
  written: the face-with-a-hole one because a ring is cut across (§36.12), and
  `MOST_PIECES_A_FACE_IS_CUT_INTO`'s because it was an `assert!` on a path
  `validate` walks — a panic on a caller's bytes, which §16.11 says it may not
  be. It is a named refusal now.
- **§35's `check_curved_face`** asks a corner for a sign and never for a
  distance, "and that is the point". It still does, and what §36.13 adds is that
  a corner carrying recorded slop is asked about the *box* it carries when the
  exact point fails. Nothing is loosened for a corner a builder placed.
- **§30.8 item 11** predicted M4 reaches `Undecidable` out of `mesh`. It does
  not, and §31.8's competing prediction is the right one. §36.14.
- **§31.3's ON row** is settled in the commit as well as in selection: two faces
  on one surface facing the same way is `intersect` plus the two orientations,
  and for a plane it stays `Plane::agreement_with` because a reversed plane is a
  different four numbers.

### 36.18 A ball cut by a plane, and the direction rule that took

Lane 5b of §31.6, and it exists because CLAUDE.md says to fix a gap rather than
defer it. §36.16 left two, both classified as gaps by the lane that wrote them,
and between them they stopped the kernel cutting a ball with a plane — which is
about the simplest curved boolean there is.

**One of the two is closed.** `Error::ACurvedFaceGoesRightRoundWithNoSeam` is
gone and §31.7 criterion 13 is met in full: `a_ball_cut_by_a_plane_meshes_into_a_closed_surface`
takes the cap above and the cap below two different cuts, meshes each through
§23.4's oracle, and checks the volume against `πh²(3R − h)/3`.

**Both poles are in that table and that is the point of it rather than
tidiness.** The lane before this one shipped an arm that never turned a step
round, and it cancelled in every test that used only curved faces; the same trap
is here, one shape along. An arm that read the pole backwards would still mesh a
*north* cap into a closed surface of exactly the right volume — it would simply
never be asked about a south one. The south rows are where getting it wrong does
not cancel, and getting it wrong is what happened first.

#### What a cap needed, and what it turned out to need

A cap's patch in the sphere's own two coordinates is a rectangle whose top edge
is the pole and whose two sides are one meridian used twice, and the model says
neither: a [`WireStep::Pole`] and a seam are what say them and a fuse can invent
neither. So they go into the mesher, four times, where a pole is a corner and a
meridian is a cut — the same move §36.12 made for a ring, with the second loop
replaced by a point.

**Which of the two poles the face reaches costs no arithmetic.** A face keeps its
material on the left of its wire, and a surface's own two coordinates are
right-handed about its outward normal — `∂u × ∂v` is `R² cos v` times it, and
`cos v` is positive everywhere but the poles — so the left of "longitude
increasing" is "latitude increasing", and a loop running that way bounds the
piece **above** it.

**And that read something arbitrary, which is the finding.** Which of a closed
edge's two coedges is the forward one says only which of the two faces along it
came first: the two have to disagree, so within one solid it is fixed, and
against anything outside it is a coin toss. §30.8 item 10 has recorded that a
closed edge has no direction rule since M3 and §35.10 called giving it one *"the
single change that widens row 10 most"*.

There already was a rule and it was half of one: *"a closed edge is stored
running the way this face's longitude increases"*, where **this face** is
whichever of the two walked it first. A bar never showed the hole in it — its two
cap circles are both between a plane and one cylinder, so both were stored the
cylinder's way and the wall's rectangle closed, which is the case the rule was
written for. A ball cut by a plane leaves two caps in **two different fuses**,
and each was stored the way whichever face got there first ran.

The rule is now: **a closed edge runs the way the longitude of the curved surface
either side of it increases, and where both are curved `Surface`'s own fixed
order picks one.** It depends on the edge and not on the walk, it keeps the bar
for the reason the old rule kept it, and the fuse sets its coedge orientations
against the same rule — `which_way_a_closed_edge_is_walked` in `assemble.rs`,
which is §36.6's identity for the wire's direction in space and one dot product
for the stored one.

**What it does not yet do is get checked.** `Model::validate` does not ask a
closed edge whether it is stored the way the rule says, so a solid arriving from
a file may disagree with it — and the builders are not asserted against it
either, because nothing they build reaches the path that reads it. That is the
half of §30.8 item 10 still open, and it is now a *check* rather than a rule.

### 36.19 The arc, and how far it got

`Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` is **still there**, and
this section is the demonstration §16.10 asks for rather than the paragraph it
forbids. It was built, it works as far as the naming goes, and it was taken out
again — so what follows is measured rather than argued, and the next lane starts
from the far end of it.

**§36.16's own row is wrong and this corrects it.** It says the half-space that
tells two arcs apart *"cannot be built here, because both ends have to be
provably on that plane once its offset is rounded and nothing rounds it in a
direction it can prove"*. All three clauses are wrong:

- **The plane is nameable without constructing anything.** Two corners on a
  closed crossing leave two arcs, and the plane through the chord *square to the
  plane the crossing lies in* meets that plane in the chord line — which a conic
  meets at exactly its two ends, so the two arcs are strictly either side of it
  whichever is the longer. The crossing's own plane never has to be built: the
  tangent `∇S × ∇T` and the chord both lie in it, so `tangent × chord` is its
  normal and `chord × that` is the one wanted.
- **Which side this arc is on is a sign and not a search.** The wire leaves its
  first corner along `∇S × ∇T` or against it, by the material side (§36.6), and
  the reading of the plane grows or shrinks along that direction. No point on the
  arc has to be built.
- **The offset can be proved rather than rounded.** `Model::validate` asks that
  both ends read *on or in front*, so the offset comes **down** — and how far
  down is not a number anybody needs, because the loop asks the predicate about
  the ends as the boxes `walk` will ask about them as, doubles the step, and
  stops when the predicate says so.

**And `Model::validate` was never the blocker either.** Lane 3 already has the
sweep skip a face trimmed by part of a curve, by name, so an answer with arcs in
it passes.

**What actually stops it**, measured: with the arcs named, a bar notched by a
block builds an answer and refuses inside `rcad_geom::walk`, which is picking
which of the two arcs to march. The refusal carries two `side of a half-space`
records about boxes round the arc's own ends — and it refuses under
`Policy::tolerant` as well as under a certified one, so it is not a budget
question. The records are swept into the refusal by a later `ask` rather than
being the decision that failed, so they name where the reading was uncertain and
not what gave up, and **this lane did not localise it further**.

So the classification is unchanged and the family of fixes is narrower by three
of its four members: it is a **gap**, the naming is done, the sweep is done, and
what is left is one function in `rcad-geom` deciding which arc a picker picks.
The two obvious members of the remaining family are that `walk`'s midpoint test
is asked about a point too near the chord for a short arc, and that
`every_half_space_holds` records a reading that *rules nothing out* — which §5
says is a route not taken rather than a decision anybody failed to prove, the
same argument `select.rs` already makes about a ray it abandoned.

**Nothing half-built was left behind.** The picker code is not in the tree: it
had no test that could pass, and CLAUDE.md's rule about that is not a preference.

### 36.20 The quartic, weighed and left

`which_comes_first_round_a_crossing` refuses a quartic, so a **ball with a pipe
through it** is still refused and §31.7 criterion 13 is met for two of its three
shapes rather than all three.

Left, and classified a **gap** exactly as §34.10 already classifies it: the two
loops are there and are tellable apart, and what is missing is an *order along*
one. That is not a sign anybody can add — a degree-four loop has no conic frame
to sweep an area in, and ordering points along one is the marcher's job, which
makes it a change to `rcad-geom`'s `walk` and `along_a_crossing` rather than to
anything this lane owns. §31.2's own table puts it at the top of the degree
column and §31.9 defers the algebra it rests on.

**It is also not what blocked the shape this lane was asked for.** A ball cut by
a plane never asks it: a plane and a ball cross in a circle, which §34.5 orders
with one determinant. Taking the quartic here would have been a second
milestone's worth of marcher work bolted to a lane whose job was a cap.

### 36.21 The numbers

Family by family and never blended (§17.7), the same four families at 384 fuses
each so the rows are comparable with §36.14's.

| Family | fuses | refused | recorded anything |
|---|---|---|---|
| lattice, a bar through a block | 384 | **0.00%** | 0 |
| turned about an awkward axis | 384 | **0.00%** | 0 |
| **a ball resting on a block** | 384 | **0.00%** | 0 |
| written at `2^k` | 384 | **0.00%** | 0 |

The ball row was **100% refused** at §36.14 and is now a control like the other
three. That is the whole of what this lane is.

| | at §36.14 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1067 | **1067** |
| suite duration | 2.9 s | 2.9 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 793 | **805** |

`just fuzz 180` ran 4 836 184 executions in 181 seconds, found **12 new seeds**
and **no crash**. Coverage stayed at 2507 edges, which is what it should be:
`rcad-geom` is untouched, so there is no new arithmetic for the corpus to reach.

**The test count did not move and that is worth a line.** A test that asserted a
refusal of the whole ball family came out and a test that meshes four caps and
checks four volumes went in. A milestone that measures itself by test count would
have scored this lane at nothing.

`just default` is green, `cargo-deny` included. **`rcad-predicates` is still at 0
files changed for the whole milestone**, and `rcad-geom` is at 0 for lanes 4, 5
and 5b together — the closed-edge direction rule landed entirely in `rcad-topo`,
because what needed settling was which of two surfaces names the direction and
not how a crossing is walked.

**No test had its number re-derived.** Nothing measured moved except the ball
family's refusal share, which went from every fuse to none.

**Did widening anything break what lane 3 proved?** Nothing in `holds.rs`
changed in this lane, so nothing lane 3 proved was widened at all. What changed
underneath it is which way round a closed edge's walked points are stored, and
§35.8's four families of point classification, `point_in_a_curved_solid.rs` and
`meshing_a_curved_solid.rs` all pass unchanged — which is the answer that
matters, because a mesh whose two faces along an edge read the same points in the
same order is exactly what §23.4's promise is made of.

### 36.22 What lanes 6 and 7 have to be told, corrected

§36.15 stands except where this lane moved it.

- **A ball cut by a plane works**, so roadmap §7.4's entries about balls reach
  what they are about rather than reaching a refusal on the way in. §36.15's
  second bullet is withdrawn.
- **The line between what is built and what is refused has moved by one word**:
  an answer whose every curved edge is a **whole closed curve** is built, and
  that now includes a face bounded by one such curve with a pole behind it. An
  *arc* is still refused, so a bar notched by a block still is, and so is any
  entry that cuts a curved face twice with crossing curves.
- **A ball with a pipe through it is refused for a different reason from a
  notched bar**, and a catalogue that runs them together will read one refusal
  where there are two: the first is the quartic ordering (§36.20) and the second
  is the arc (§36.19).
- **The closed-edge direction rule is a rule and not yet a check.** A metamorphic
  property that builds a curved solid, writes it out and reads it back is fine;
  one that hand-builds a closed edge with the coedges the other way round will
  not be caught by `validate` and may mesh a cap to the wrong pole. Say so rather
  than assume it.

### 36.23 The arc, closed, and what was really in the way

§36.19 measured how far the last lane got and named two candidates for the rest.
**Neither was the cause**, and the localisation it asked for is one sentence: the
naming was right, the walk was right, and what threw it away was the
**coplanar joiner**.

An arc built by `Edges::of` carried the half-space that named it. Then
`join_up_what_is_one_face` ran, joined runs of arcs into one edge at every corner
only two edges leave, and handed the joined edge ends the half-space had never
been built for. §36.19's two candidates — a midpoint too near the chord, and a
picker's `Undecidable` ruling nothing out — are both about *reading* the
half-space, and the half-space that reached `walk` was for a different pair of
ends. Measured: a bar notched by a block joins exactly two arcs into one, twice,
and both of them are on the bar's cap circles where the block's planes cut them
and nothing else does.

**Both of §36.19's candidates are still true statements and neither fires.** With
the joined arcs named, the four families are at 0.00% and the notched bar records
nothing at all.

#### What names an arc, and why nothing on it is built

The half-space is the one through the chord **square to the plane the crossing
lies in**: that plane meets the crossing's plane in the chord line, a line meets
a conic at exactly two points, so the two arcs are strictly either side of it
whichever is longer.

Neither plane is ever constructed. For `v` a direction from one end **into** the
arc and `c` the chord, both lie in the crossing's plane, so `v × c` is that
plane's normal and `c × (v × c)` is the one wanted — which written out is

    facing = v(c·c) − c(c·v)

and whose dot with `v` is `|c|²|v|² − (c·v)²`, never negative by Cauchy–Schwarz.
**So the normal already faces the way the arc goes and there is no sign to case
out.** §36.19 wrote this with `v` the tangent `∇S × ∇T`; the generalisation to any
`v` into the arc is what makes joining work, and it costs nothing.

Which way the wire leaves its first corner is a sign and not a search: left of
`∇S × ∇T` is always the partner's negative side (§36.6), so it leaves along that
direction exactly when its material is inside the partner — which
`FaceMeet::material_side` already holds, written from which way round the
arrangement walked the arc and costing no arithmetic.

The offset is proved and not rounded, exactly as §36.19 said: it comes down from
the smaller of the two `f64` readings by a step that starts at one rounding of
the numbers involved and doubles, until the predicate itself says both ends read
*on or in front*. It stops once the total is past twice `|facing| · max|p|`,
which Cauchy–Schwarz puts past any reading there is, so a half-space not found by
then does not exist.

#### The joined arc, and the corner that is a point on it

A run of arcs joined into one is named the same way any one of its pieces is —
and **a corner the joining dissolved is a point on the joined arc**. So `v` is the
direction from a surviving end towards one of them, and still nothing is built:
that corner is one the arrangement placed.

An unjoined arc uses its own stored direction, which is the tangent. Two arms,
and both are walked by the shapes below: every arc of the notched bar's own
notch is unjoined and every arc of its cap circles is joined.

#### What is left of the refusal

`Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` is still there and is now
one case: `facing` with no length, which means the chord lies along the
crossing's own direction. For a conic that is an arc so short the chord rounds
onto its own tangent — two corners a hair apart on the unit circle, where
`cos δ` is exactly one and `v(c·c) − c(c·v)` cancels to exactly nothing. A
**gap** (§16.10): the half-space exists and `f64` cannot reach it.
`an_arc_whose_chord_rounds_onto_its_own_tangent_is_refused_by_name`.

#### Two things §36.19 did not see

**A straight piece of a crossing is not an arc and must not be named as one.** A
block's `y = 1` plane cuts a bar's wall in two straight lines, and those are edges
between a plane and a cylinder with two different ends — which the old table sent
to the refusal. They are `EdgeRun::Straight`, whose own doc has always said
"straight between its two ends, which are the whole of it", and `walk` has always
had nothing to put between them. **The test is read off the surfaces and never off
the chord**, in the same three cases and the same spelling `walk` uses, because a
short enough arc has a chord that rounds onto its own tangent and this question
has to stay one about the pair.

That is also worth more than it costs: `holds.rs` refuses a wire with any
non-straight coedge, `classify.rs` fans a face whose every side is straight, and
`view.rs` widens a flat face's box for every side that is not. Calling a straight
side straight is right in all three.

**Which surface goes in `and` is decided by a check and not by taste.** The field
names the second surface for a run whose two faces are the *same* surface — a
ball's seam — and an arc's two faces already name both, so either is true. What
picks between them is that `Model::validate` asks both ends to read exactly `Zero`
against `and`, and a corner this fuse solved for is exactly on a plane it was
solved from and only within its own box on a quadric. The flat one goes in, and
`name_of_surface_bits` gives that for free by sorting a plane ahead of a ball or a
pipe.

A crossing with **two** curved surfaces and two ends is a pair of balls cut twice,
and nothing builds one: measured, two balls united refuse at
`ACurvedFaceGoesRightRoundWithNoSeam` long before an arc of them is named. When
that stops being true what has to give is the *check* — the same slop
`check_curved_face` already reads off a solved corner (§36.13), read in
`check_edges` too — and not the choice. Lanes 6 and 7 are told.

### 36.24 The direction rule stops being a rule, and the two things it was wrong about

§36.22 said the closed-edge direction rule was a rule and that `Model::validate`
did not ask a closed edge whether it obeyed it. It does now, with the very
function the fuse chooses by — `a_closed_crossing_is_walked_the_way_it_is_stored`
— because two spellings of one question is how those two come to disagree
(§23.8). `a_closed_edge_walked_the_wrong_way_round_its_surface_is_caught`, with a
control that flips *one* coedge and is caught by the older check instead.

Reading it back found two things the rule as written in §36.6 got wrong.

**The face's own orientation does come into it.** §36.6 says it does not: "a wire
winds about the surface's stored normal whichever way round its solid uses the
face". A sphere and a pipe have no reversed form, so a curved face the rule turned
round is *the same surface* with its wire stored walked the other way and
`FaceUse::orient` saying so. The rule has to be asked about the gradients the
**solid uses**, and compared against the coedge's turn **as stored** — the turn a
view composes back is the un-turned one. Both halves are load-bearing: the
block-with-a-round-hole answer fails the check on either one alone.

**`a_washer` had its hole's circles walked backwards**, both coedges of each, and
the comment in the fixture argued that it should. It survives every other check
because a wire that is one closed curve has no polygon to wind, so the outline and
the hole ran the same way round the cap and nothing looked. A hole runs the other
way round a face than its outline does, and this is the first thing in the tree
that can say so. §36.22's own warning — "may mesh a cap to the wrong pole" — was
about a property nobody had written, and the thing it warned about was already in
the tree.

**A ball turned inside out is still caught by the volume and not by this.** A ball
has no closed edge at all: its one seam runs pole to pole. So
`a_curved_solid_turned_inside_out_is_caught` is a two-row table now, and §23.9's
own claim — a negative meshed volume — is measured in it rather than waited for.

### 36.25 What each criterion in these two lanes cost

| | | |
|---|---|---|
| **13** | a fused curved solid validates, meshes and passes §23.4's oracle | **met for all three shapes in this lane's reach**. `a_bar_notched_by_a_block_meshes_into_a_closed_surface`, beside §36.18's ball cap and §36.12's block with a round hole. A ball with a pipe through it is still the quartic (§36.20) |
| **16** | the four families at 0.00% | met, unmoved, and the notched bar joins them as a fifth shape that decides everything and records nothing |
| §30.8 item 10 | a closed edge has no direction rule | **the rule became a check** (§36.24), and it found a fixture that had been wrong since it was written |

### 36.26 Every refusal these two lanes can produce, classified

§36.16's first row is **withdrawn**: it says the half-space that tells two arcs
apart cannot be built here, in all three of its clauses, and §36.19 already
corrected the clauses. What it refuses now is one case and not the shape.

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` | the chord of an arc lies along the crossing's own direction | **gap** | `an_arc_whose_chord_rounds_onto_its_own_tangent_is_refused_by_name`. The half-space exists; `f64` cannot reach it at that separation |
| `Invalid::AClosedEdgeIsWalkedAgainstItsSurface` | a face walks a closed edge against the way its surface winds | **not a refusal about arithmetic** — it is the sweep saying the model's topology and its geometry disagree, which §16.11 says arrives from a file |
| everything §36.16 lists apart from its first row | | | unchanged |

### 36.27 The numbers

Family by family and never blended (§17.7), the same four families at 384 fuses
each so the rows are comparable with §36.21's.

| Family | fuses | refused | recorded anything |
|---|---|---|---|
| lattice, a bar through a block | 384 | **0.00%** | 0 |
| turned about an awkward axis | 384 | **0.00%** | 0 |
| a ball resting on a block | 384 | **0.00%** | 0 |
| written at `2^k` | 384 | **0.00%** | 0 |

Not one of them moved, which is the point: an arc that used to refuse by name
appears in none of the four, and naming it broke none of them.

| | at §36.21 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1067 | **1070** |
| suite duration | 2.9 s | 2.9 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 805 | **815** |

Three tests net: the notched bar's mesh, the direction-rule check, and the arc
refusal's own demonstration. A fourth changed hands — the test that asserted the
notched bar was refused by name now asserts it is decided and records nothing.

`just fuzz 180` ran 5 210 761 executions in 181 seconds, found **10 new seeds**
and **no crash**. Coverage moved by one edge, 2507 to 2508, and that is a longer
run rather than new arithmetic: `rcad-geom` is untouched by these lanes.

`just default` is green, `cargo-deny` included. **`rcad-predicates` is still at 0
files changed for the whole milestone**, seven lanes now, and **`rcad-geom` is at
0 for lanes 4, 5, 5b and 5c together**. That is worth a line of its own: §36.19
said what was left was "one function in `rcad-geom` deciding which arc a picker
picks", and the answer was in `rcad-topo` — the picker was right and the joiner
threw it away.

**The notched bar, measured.** A bar of radius 2 and height 8 with a 2-cube
notched out of its wall meshes into 436 triangles at a deflection of 0.02, closes
up under §23.4's oracle in full, and measures **96.8665** against the
**96.8781** the closed form gives — 0.012% under, and under is where an inscribed
mesh belongs. Refined ten times it comes to 96.8769, which is 0.0012%.

**And the shape where the error would not cancel.** Every arc a block leaves on a
bar of that size is shorter than half a turn, so a picker that always took the
shorter of two would pass the notched bar and never be asked — the same shape of
hole as the north cap that meshed correctly with the pole read backwards
(§36.18). A wedge with its point at `(−1, 0)` opening in `+x` leaves an arc of
**228 degrees**, joined out of three stretches, longer than its own sibling. It
measures **87.6998** against **87.7111**, 0.013% out. Both rows are in one test
and the second is why the first is worth anything: with the direction into the
arc read backwards, **both** shapes stop being solids at all, refused by the
triangulator for a boundary that crosses itself.

### 36.28 What lanes 6 and 7 have to be told, corrected again

§36.22 stands except where these two lanes moved it.

- **A bar notched by a block works**, so the line §36.22 drew at "a whole closed
  curve" has moved: an edge along *part* of a curved crossing is built and named.
  Its third bullet is withdrawn.
- **The closed-edge direction rule is a check now**, so §36.22's last bullet is
  withdrawn too — and the thing it warned about was in the tree, in `a_washer`,
  and had been since that fixture was written. A hand-built curved solid is worth
  running `Model::validate` over before it is trusted, and two of the ones in the
  tree were not.
- **A ball with a pipe through it is still the quartic** (§36.20), and it is now
  the *only* shape in roadmap §7.4's reach that a curved fuse refuses for a reason
  of its own.
- **`check_edges` asks an arc's ends for an exact `Zero` against `and`.** That is
  what makes the flat surface of the pair the one stored there (§36.23), and it is
  a check a pair of *two* curved surfaces would fail on a corner that is only
  within its box. Nothing builds such a pair today. Whoever makes two balls
  fuseable closes `ACurvedFaceGoesRightRoundWithNoSeam` first and inherits this on
  the same day.
- **The coplanar joiner reaches curved edges and is written in planes.** It joined
  arcs of a cap circle correctly here only because this lane taught the join to
  re-name what it made. Anything else added to `Edge` that depends on its two ends
  has the same problem and will not announce itself.
- **A notched bar works on the lattice and a turned one does not**, and the
  reason is not the arc. Measured over twelve rotations about `(1, 2, 3)`, every
  one of them refuses inside the *arrangement* with `Undecidable` naming **order
  along a crossing** — which runs before anything names an edge, and refuses
  under `Policy::tolerant` as well as a certified one, so it is not a budget
  question. §34.5's determinant orders three points round a conic and a turned
  cap circle cut at seven places is where it stops. The turned family in the
  table above is at 0.00% because its bar passes *through* the block a whole unit
  clear of every wall and its cap circles are cut nowhere.
- **`select.rs` cannot place a point inside a flat piece bounded by one chord and
  one arc.** Two corners leave no polygon to wind and every candidate is rejected,
  so a bar cut by a single plane not through its axis refuses with `Undecidable`
  naming *a point inside a piece of a face*. §36.16 already lists that refusal;
  what is new is that arcs make it reachable from a shape as plain as that. A
  **gap**, and what closes it is a piece that is read by which side of each of its
  bounding surfaces it lies on rather than by winding — which is the rule
  `the_piece_holds` already uses for a *curved* piece, one dimension across.

