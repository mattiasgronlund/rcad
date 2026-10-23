## 220. The new closed run reaches the mesher, the invariant and the validator, and a property finds the pipe — 2026-09-24

§0's next unclaimed row and the three §219.8 items beside it, one lane each. Four
lanes off `64b83b31`, three writing and each verified by a Sonnet reader that
reran every number, one test-only and unverified by design.

`just gate quiet` on the merged tree `e524637c`: **PASS, 2342 tests run, 2342
passed, 7 skipped**, 25 doc tests across 9 crates. The base `64b83b31` was gated
by the dispatching session before any lane was cut — PASS at 2328, 6 skipped.

All three §219.7 items are closed. What this batch adds to the open list is
smaller than what it took off it, and the one new thing is a wrong answer with
its test already written and its fix already named — §220.4.

**§219.7's producing chain is wrong as written, and §220.1 is the correction.**
That is the part of this section to read if you read one part.

### 220.1 The pole, and a chain that does not run

`mesh.rs:738` gated pole insertion on the old closed variant by name, so a rim
written §216's way took the `else` and no pole was guaranteed among the walked
points. It now runs for both kinds, and **the pair it hands over for the new kind
is the run's own `[along[0], and]`** and not the two faces — which for that kind
are the one surface twice, and read as a ball crossed with itself. That is the
same pair `rcad_geom::walk`'s `the_two_surfaces_it_runs_between` already reads
off the run, so this is not a second answer to §23.8's question.

The test is a ball of radius 5 split down its polar axis into two lunes, built
through `SolidBuilder`, the rim between them written `RightRoundWhereItCrosses {
and: Plane(x = 0) }` with its one corner a quarter turn from either pole. Seven
deflections, both walk directions.

**On the base it is a refusal at five of the seven deflections and not a silent
wrong mesh** — `ACurvedFaceGoesRightRoundWithNoSeam` — and at the two that
answered, the walk had landed on the pole by itself. With the change all seven
answer, and every one has a mesh point exactly on the north pole, 0 away. The
volumes run 467.11215906329 at the coarsest to 523.2627879331797 at the finest
against the closed form's 523.5987755982989. All of that is from a printing probe
in the test file under `mise exec -- just t -E
'test(/^a_closed_rim_the_new_way_still_gets_its_poles::/)' --no-capture`; the
verifier reverted the source to base and reproduced the quoted failure to sixteen
significant figures.

**§219.7 said this shape comes out of `Model::fuse` and named the chain end to
end. It does not, and the lane measured why.** Two separate obstacles:

- `fuse/cuts.rs:106` returns early on `if !solids[one].bounds.overlap(solids[other].bounds) { continue; }`. A ball of radius 2 at the origin united with a block whose `x = 0` plane holds the polar axis came back with the sphere face uncut, one face and only its seam.
- Even with the boxes overlapping, a plane holding the ball's polar axis cuts the great circle through **both** of `add_ball`'s pole vertices, so the fuse splits it into two arcs and it is not a closed rim at all. Any plane through the axis does this.

So the sentence in §219.7 is wrong about `add_ball`'s ball. It may hold for a
sphere face the fuse has already rebuilt without a seam; nobody has reached one.
The lane reached the same face pair through `SolidBuilder` instead, which is the
public door §216 already says this run kind must pass through.

**This is §16.10 read the other way round.** §219.7's chain was a claim that
something *can* be reached, written as a paragraph rather than a demonstration,
and it was wrong in the same way an unreachability paragraph is wrong. A named
route needs a demonstration too, and the memory to keep is [[a-named-fix-in-a-
section-may-be-unmeasured]]'s: §181.6's "it would work" was never run either.

**Two more findings from that lane are its own and were not reproduced**, and the
verifier says so. A cap of a ball bounded by one circle through a pole, with the
corner *not* at that pole, is refused `ADraftFaceCouldNotBeMeshed { why:
CornersCoincide }` — and identically for both run kinds, so pole insertion is not
what is missing there; it looks like §199.4's family with one case left open.
And a half ball whose skin carries a hole through the north pole is refused four
different ways across the four coedge orientations, all downstream of pole
insertion, which did fire. Neither is in the tree and neither is confirmed.

**2026-10-04: the half ball is confirmed and landed.** Q582, `de799a3b`
(`mesh.rs`'s commit is `2eca0ea0`). `cut_a_ring_across` now gives the band a
corner wherever a pole side passes a quarter, and breaks a tie toward the
corner off the pole. All eight right spellings mesh closed and measure half
the ball, pinned in
`a_half_ball_holed_through_its_pole_and_capped_on_the_same_ball.rs`. The cap
finding above is still unreproduced.

### 220.2 One list instead of two spellings kept in step by hand

§219.7 item 2, one gap in two places. `NamedRun::new` checked
`run == RightRoundWhereItsFacesCross && ends[0] != ends[1]`, although its doc
claimed the check for a closed run generally; `where_a_plane_cuts_a_cone` had the
same old-variant-only test, so a new-variant run with `leaning == 0.0` came back
`Ok(Parabola(..))` — open and unbounded — to a caller that had asked
`is_closed()` and been told yes.

Both now read one list: `EdgeRun::it_comes_back_to_where_it_started`, new and
`pub(crate)` in `meet.rs`. Two tests fail on the base, one per site, and two more
are controls — an open run still takes two different ends, and an old-variant
closed run gets the parabola refusal it always did.

**The five other sites in `walk.rs` naming only the old variant were judged and
none was changed.** Four are right by reading: `:913`, `:2300` and `:2721` name
all four variants and put both closed kinds in the same empty-slice arm, which is
correct because neither closed kind carries a `picked_by`; `:1629` is the
fall-through of `the_two_surfaces_it_runs_between`, whose new-variant arm sits
twenty lines above it.

The fifth needed more than reading and got a demonstration. `:726`'s guarded arm
is `[Cone(one), Cone(other)] if one == other && run.run() ==
&RightRoundWhereItsFacesCross`. The §216 shape proper arrives there as `[Cone,
Plane]` and goes to site 2, so the only way a new-variant run reaches `[Cone,
Cone]` is by naming the face's own cone as its `and` — saying the second surface
is the surface that already named nothing. It then refuses
`ACrossingBetweenAConeAndAnotherCurvedSurface`. A refusal, no curve there to
build a circle through, so widening the guard would be wrong.
`a_second_way_run_whose_second_surface_is_the_cone_itself_refuses` drives the
public `walk` and shows it.

**The lane's grep counts are wrong and are not repeated here.** It reported seven
matches in `meet.rs` and fourteen in `walk.rs` at the base; the same command on
the same sha gives nine and thirteen. The verifier checked the per-line judgments
separately and they are accurate, so this is a miscount in the prose and not a
wrong analysis — but a number that does not reproduce does not go in the record.
§45.2's rule is about counts that are quoted, and the cheapest way to obey it is
to quote none of them.

### 220.3 The validator reads the run, and says so about the second spelling

§219.6 and §219.7 item 3, both in `validate.rs`, on one commit.

`check_this_solids_closed_edges_are_walked_the_way_their_surfaces_wind` now
builds its pair with a new `what_says_which_way_this_closed_edge_winds`, which
swaps the run's `and` into slot 1 for `RightRoundWhereItCrosses` when
`the_curved_one_of` says neither face is curved.
**`a_closed_crossing_is_walked_the_way_it_is_stored` keeps its signature**, so
`assemble.rs` and `mesh.rs` are untouched — which is what let this lane run
beside §220.1's. And `RightRoundWhereItCrosses` no longer falls into the
open-edge wildcard at `:403`: both closed kinds now name
`ClosedEdgeBetweenTwoFlatFaces`.

§219.6's standing demonstration became the passing case it said it would. Two
spellings of one plane a hair apart now import, validate and mesh.

**The corpus did not move at all** — not the rates, not one file's refusal name —
so `how_often_a_real_file_imports.rs` was in the lane's write set and needed no
edit. This fix is covered by two hand-written tests and nothing in the 367-file
corpus reaches it. §45.2 wants that stated, and "no file moved" is the statement.

**§23.8 is now spelled twice, deliberately and narrowly, and the doc comment says
so.** Picking the pair here *is* the question `rcad-geom`'s
`the_two_surfaces_it_runs_between` answers. The new one fires only on
`RightRoundWhereItCrosses` and only when `the_curved_one_of([face, across])` is
`None` — exactly the condition the callee refuses on — so the two cannot disagree
by widening. The geom one answers `[*along[0], *and]` for that run kind
unconditionally, which is **order-dependent where `and` repeats one of the two
faces**: a sphere face and a plane face with `and` the plane gives `[plane,
plane]` one way round and `[sphere, plane]` the other. Closing this properly
wants one public answer in `rcad-geom` that both read. That file was outside the
lane's write set, so the argument went into the doc comment rather than a second
authority going in quietly.

### 220.4 The property found the pipe, and it is the next row

§219.8 asked for coverage over spellings of one surface and got it:
`every_spelling_of_one_surface_names_the_same_crossing.rs` draws a surface on a
lattice, re-spells it one to four ulps away, hands both spellings to `walk` and
`where_a_closed_crossing_starts` beside a run naming a second surface, and
demands the answer be **bit-identical** to the pair written twice bit for bit.

Exact equality is the right oracle and not a tolerance, because every arm that
fires hands back `[along[0], and]` — the walker is handed literally the same two
surfaces, so anything but bit equality would be slack with nothing to spend it
on.

**The fallthrough is what proves the property is not a tautology.** All four
closed arms agree, 0 disagreements. The fifth bucket runs the identical
comparison over pairs no arm catches and **disagrees 405 of 592 times**. The four
green arms are green because the arms hold, not because the comparison is blind.

**The defect, and it is a gap.** §189.3's arm reads `both_curved &&
and.as_plane().is_some() && a_ball.is_some()` — it asks for a **ball by name**.
A pipe, a cone or a torus written twice a hair apart, whose run names the plane
its rim lies in, matches no arm and falls through to `[*along[0], *along[1]]`:
two nearly-coincident curved surfaces instead of the surface and its plane, and
it loses its rim. The rim exists, the bit-identical spelling walks it, and the
arithmetic reaches it, so §16.10 makes this a gap and not a limit. The fix is to
widen that guard from "a ball" to "either face", which is what §219.1 already did
one row down for the curved `and`.

`a_pipe_written_twice_a_hair_apart_walks_the_circle_its_plane_names` asserts the
right answer and is `#[ignore]`d. **The seventh `#[ignore]` is why `justfile`'s
`ignored_tests` moved from 6 to 7, and that bump is not a lane's to make** — the
count is the gate's and not a crate's, so a lane forbidden the gate cannot see
it. It landed with the merge. [[a-lane-forbidden-the-gate-cannot-see-gate-only-
invariants]] is the same shape a third time.

Two things the property does not cover, for whoever widens it: `and` repeating
one of the two faces bit for bit when the pair is one surface written twice — 11
draws, skipped as malformed input — and the pair written twice at very large or
very small magnitudes. Every draw here sits on the lattice −4..4 with radii 1, 2
and 4.

### 220.5 What this leaves

Off the list: all three §219.7 items, and §219.8's "no property covers spellings
of one surface".

Newly open:

- **§189.3's arm asks for a ball by name** (§220.4). A wrong answer, a gap, the
  test written and `#[ignore]`d, the fix named. This is §0's next row.
- **§23.8 spelled twice across two crates** (§220.3), the narrower one documented
  as such. Wants one public answer in `rcad-geom` that `rcad-topo` reads.
- **§219.7's producing chain needs a real route or a correction in place**
  (§220.1). The correction is here; a route from `Model::fuse` to a closed rim
  between two faces of one sphere is still nobody's demonstration.
- Two unreproduced findings from §220.1 — a ball's cap whose corner is off the
  pole, and a half ball with a hole through the pole. Neither is in the tree.

Still open from §219.8 and untouched by this batch: `round_run`'s `==` misses two
curved faces a hair apart at the importer's door; `mesh.rs:5148`'s oracle carries
the bit equality §219.1 fixed; `arrange.rs:4489` respells `n·a`; `holds.rs`'s
`is_a_family_this_follows` is a gap since §208.6.

### 220.6 A rim cornered on the pole: the use is a label, and a lid is a disk

*Queue row Q941, which was Q651. Lane `lane-q941-a-closed-rim-cornered-on-a-pole`,
off `26086a58`. 2026-10-06.*

**The question.** §220.1's half ball holed through its pole and capped on the
same ball, its rim cornered on the north pole `N` and named by the plane
`y + 3z = 15`, builds, validates and measures half the ball in all four ways of
writing it — the plane either way round, the skin's use of the hole either way
round. Q582 pinned two as right and called the other two wrongly walked, and the
row asked which they are: two spellings of one solid, or a wrong spelling
`Model::validate` lets through.

**Two spellings of one solid, and the row's premise was wrong.** The row said
the mesher keeps the walker's own order on this rim. It does not: the ball's
longitude and latitude both vanish at `N`, so the rule reads nothing and
validate skips the edge, but §341.5's reading answers — the skin's corner on the
equator is on the plane's negative side, so its use walks the rim along
`outward × ∇plane`, and the edge runs the way that use says. The skin's use is
then a label for the edge's direction and not a claim about the shape. Measured
in `the_four_ways_of_writing_the_rim_on_the_pole_are_one_solid`
(`a_half_ball_holed_through_its_pole_and_capped_on_the_same_ball.rs`), on the
base and after this lane alike, under `Policy::certified()`, in both wire
orders:

- the skin holds `(0, −0.3)`, `(0.3, 0)` and `(−0.3, 0)` beside the pole, off the
  cap, and holds neither `(0, 0.3)` nor `(0, 1.5)`, on it — in all four;
- the cap's own answer is one refusal in all four (below);
- the two uses of one plane mesh **the same triangles**, compared as the three
  corners of each in space, bit for bit, with its winding;
- every skin triangle is short of the plane and every cap triangle beyond it;
- the rim's mesh points leave `N` towards `+x` where the skin uses the hole
  `Reversed` and towards `−x` where `Forward`, whichever way the plane faces, so
  the skin walks away from `N` towards `−x` in all four.

Volume would not have told: a solid walked the wrong way round can measure
right. What each face holds is the same, so it is one solid.

**Which direction defines the stored direction at a pole corner: neither of the
two the row offered, and §341.5's stands.** Where the rule reads nothing the
edge's stored direction is not a fact about the shape, and the way each face
walks it is.

- *The walker's own order* is what §341.5 replaced, and for the reason §341.4
  measured: it turns round with the way the rim's plane faces, so which of two
  writings of one solid builds would depend on the plane.
- *The longitude at the walk's first point* is a fact on a circle through one
  pole — the longitude grows the same way all round it off the pole — but not on
  a great circle through both, where the longitude is square to the circle at
  every point. §341.5's half ball on a pipe is that shape. It would also have
  validate refuse writings measured above to be one solid, and read the
  direction differently from the four other callers of
  `a_closed_crossing_is_walked_the_way_it_is_stored` (the fuse, `bake.rs`,
  `holds.rs`, the mesher's oracle), which all read `Zero` there: a second
  authority for one question (§23.8).

**So validate is given no new direction and keeps skipping the edge**, on
purpose: against a label there is nothing to check a use by. What checks the
faces' walks against each other on the pole is the sweep's mesh, and it does:
the same half ball with a flat lid in place of the cap builds with the skin's use
either way round and measures the same, and with the lid's plane turned to face
into the solid it is refused `FacesDoNotAgreeWhichWayTheyFace` in every spelling
(this lane's probe, not kept).

**A lid is a disk, and that is what moved.** Q606's ball less a cap, and the cap,
lidded by a flat face and cornered on `N`, have no corner off the rim on either
face, so §341.5's reading said nothing and the walk kept the walker's order. One
of the two lid uses built and the other was refused
`FacesDoNotAgreeWhichWayTheyFace`, and which one turned with the lid's plane —
§341.4's complaint again, on a shape §341.5 did not reach. A flat face whose one
wire is the whole rim holds the disk inside the ball across it, which is a fact
with no corner needed, so its use walks along `outward × ∇ball`. `walk` reads that
where the faces' corners say nothing (`e4afc595`), and both lid uses now build,
mesh closed and measure the closed form at three deflections and after a union
(`a_rim_cornered_on_the_pole_builds_either_way_round_and_both_ways_measure`, which
fails on the base). A lid turned to face into its piece is not a third case: by
construction its faces are then exactly the other piece's — the same plane, the
same ball, the same uses — and the probe found it builds.

**What this leaves.**

- The cap of the holed half ball refuses `face_holds_the_point` by
  `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` at every point asked, on the
  pole only: its one corner is on the plane, and the wire's side
  (`the_side_its_wire_keeps`, `holds.rs`) reads the shared rule, which says
  nothing there. The same cap off the pole answers. A gap: the cap's side is a
  fact, and §341.5's reading off the skin, or the disk's, would give it.
- The disk reading sits inline in `walk` because the lane held `mesh.rs` only as
  that function; it belongs beside `the_way_this_face_walks_a_closed_edge`.
- Q630's spelling (the lid listed first, the rim named by the lid's plane turned
  over) is still refused `TooFewCorners` both ways round, and is still left out
  of the pin.
