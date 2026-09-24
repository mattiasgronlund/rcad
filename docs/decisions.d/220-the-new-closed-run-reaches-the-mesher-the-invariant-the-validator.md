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
'binary(a_closed_rim_the_new_way_still_gets_its_poles)' --no-capture`; the
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
