## 198. The turned ball-and-block measures, a pole within a rounding is the pole, and one half ball cut again answers — 2026-09-22

*One batch, dispatched by §175's skill off `52e4766d`: workflow run
`wf_aec3f0fa-293`, two Opus lanes, each checked by a Sonnet verifier. `just
gate quiet` run here: on `52e4766d` before the batch, PASS, 2230 run, 2230
passed, 6 skipped; on the merge `d4b2e906`, PASS, 2235 run, 2235 passed, 6
skipped. A number below is a lane's unless it names a command run here. The
verifiers re-ran the numbers they could and found none that disagreed; the
ones they could not re-run are named below.*

### 198.1 A circle that passes within a rounding of a pole gets the pole, and the walk calls a point that close the pole

`lane-ball-block-poles` (`046ca9c4`) closes §197.1. It made three changes to
`mesh.rs`:

1. `put_in_a_pole` also puts in a pole when the circle where the ball meets the
   plane passes within one `POINT_ALONG_A_CROSSING_ROUNDOFF` of it. 2d1e5077's
   rule (put the pole in when `atan2` reads the circle's nearest point as the
   pole) missed `a8b648d9`'s north pole.
2. The wire walk calls a corner or walked point within that same distance of a
   pole the pole. A closed edge that is a single wire step is then no longer
   read as a seam turning back, and the loop closes over the pole side.
3. `finish` decides "right round" from the walk's own longitude, as 2d1e5077
   did.

**Why the three tries §197.1 reverted broke the `[0, 1, 0]` sweep.** At radius
1 that circle starts 4.4e-16 from the south pole, and `atan2` reads its
latitude one ulp short of −π/2. Putting the pole in beside the start gave two
corners that the flat face reads as one (`CornersCoincide`). Leaving the pole
out, the walk's own winding read the −π swing correctly and called the face a
cap through the north pole, so the faces disagreed. The old short-way sum only
passed because it happened to read the north pole's side as +π. Change 2 fixes
this by walking the start as the pole. The ablation (2d1e5077's `mesh.rs`
alone answers 24 of 36, and change 2 without change 1 also 24 of 36) ran on a
probe switch that was not committed, so it is the lane's word.

§197.1's proposal, deciding the winding from the exact side of each pole
against the plane, was not needed and was not built.

Evidence, all re-run by the verifier: four regression tests in
`a_turned_ball_and_block_that_would_not_measure.rs`, one per axis (−1,−3,0),
(3,−2,0), (1,3,0) and (3,1,0), each measuring against the closed form, and each
failing with `mesh.rs` put back to the base. The sweep answers 36 of 36. Seeds
`7d4e1f9a`, `a8b648d9` and `9a699edb` are committed in the regressions file.
The (1,3,0) seed's hash was never kept, so it has a named test but no seed.
`9a699edb` is the (3,1,0) case, which a 2000-case hunt drew on the base tree.
At 2000 cases on the fixed tree the property passed: 12108 fuses, 48 refused
(0.4 %), 0 not a solid, and 333 (2.75 %) where the mesher refused a face of the
answer by name, which the property accepts. That run needed a temporary edit
to the case count, so nobody but the lane has seen it. The base's rate of
mesher refusals was not measured, so 2.75 % is not a before-and-after.

Side effects, both pinned: all four knife cuts that
`a_narrow_lune_from_a_pole_corner.rs` listed as refused now answer, each a
valid solid measuring half the ball, so both of its refused lists are empty.
§197.3's cases no longer reach `mesh.rs:839`, because both poles are now put
in. The piece that holds both poles is still not built. **`LED_D5.0mm-3.step`
now imports** (KiCad 160 → 161, `just t -p rcad-step` here). That is the file
§115 left carrying the closed-edge reason. Which of changes 1 and 2 freed it is
not measured, and its solid is checked by `Model::validate` and nothing else.

Still open: `runs_up` still adds corners the short way round, and its doc
comment now says so. For a cap whose rim passes through the other pole it is a
coin toss, and no test reaches that case.

### 198.2 A half ball cut across its polar axis measures, and the other two pins move one blocker on

`lane-half-ball-cells` (`2cc2d002`) took on §197.2's three pins and answered
one of them: `a_half_ball_cut_across_its_polar_axis_measures_half_a_ball_without_its_cap`
now validates and measures against its closed form. The lane found four places
where the answer got lost, each with printed probes:

1. `arrange.rs`: the z = 0.5 circle starts exactly where the x = 0 rim crosses
   it. The node there is a box one ulp wide around the start, so all three
   points lie on one line, the proved `Zero` read as "one point", and the two
   crossings merged. `the_one_the_start_is` now decides it from which box
   misses the start, and returns `Undecidable` when both boxes miss it or
   neither does.
2. `select.rs`: a half disk has two corners, so its corner polygon winds round
   nothing and every candidate sat on the chord. A point between a ball-cut
   side and its chord now counts, with one new candidate on each side of every
   curved side.
3. `assemble.rs`: the chord and the arc between the same two corners got one
   edge name. The key now says whether the edge is straight.
4. `assemble.rs`: the two pole-to-pole arcs of one circle where a ball meets a
   plane also got one name. Each is now told apart by which way it leaves the
   lower-numbered corner.

Fix 4 moved one refusal outside the lane's write set, and it was re-pinned at
the merge: `fuses_that_touch.rs`'s ball resting on a block it overhangs is now
refused `AWireWillNotLieFlatOnItsFace`, where it was `ResultIsNotManifold`. The
lane showed with a probe switch that fix 4 is what moved it. It is still a gap.

The other two pins still refuse, one step further on each:

- **x ≤ 0.5** now passes the fuse and is refused in `mesh.rs`
  (`CurvedFaceWithAHoleGoesTooFarRound`, where `cut_a_ring_across` gives up).
  The skin's outer wire is the x = 0 great circle, as two edges from pole to
  pole, and its hole is the whole x = 0.5 circle. The fix needs a slit from the
  hole's one corner to a pole, as `open_a_slit` makes for a single wire.
- **The union twin** is now `Error::Undecidable`. `holds.rs`'s
  `what_a_round_wire_encloses` wants a hole wire's first edge to be closed, so
  a hole walked as two edges from pole to pole is refused. The new test
  `the_flat_face_of_a_ball_joined_to_a_block_down_its_polar_axis_cannot_be_followed`
  pins that, and will fail the day `holds.rs` treats that wire as round, which
  `select.rs`'s `round_wire_of` already does (§194).

Not done, on purpose: the same fix for pipes, whose cuts are ellipses. The lane
tried it. Two pinned refusals of a bar in `degenerate_curved_fuses.rs` started
to answer, two more became `ResultIsNotManifold`, and the ratio test in
`how_often_a_fuse_with_a_cone_refuses.rs` broke. It needs fix 4 widened to
pairs of a plane and a pipe first, and the `in_the_bulge_of` doc comment says
so. The same comment's reason for leaving cones out ("a plane can cut a cone
in two branches") has nothing behind it. The verifier flagged it as prose
where §16.10 wants a demonstration, and it stays flagged here.

### 198.3 What is left

- `mesh.rs`: cut a curved face whose outer wire is a great circle cut at the
  poles and whose hole is one closed curve (198.2, x ≤ 0.5).
- `holds.rs`: a hole wire of two edges along one ball is round (198.2, the
  union twin). It writes a different file from the item above, so the two can
  run side by side.
- `mesh.rs:839`, the piece of a ball holding both poles (§197.3). No test
  reaches it any more.
- `runs_up`'s short-way sum for a cap whose rim runs through the other pole
  (198.1).
- The narrow lune, and the named refusal in `mesh.rs` for a closed edge that
  neither direction rule answers, both still open from §196.4.

Commands: `just decision 197.1`, `197.2`, `197.3`, `115`, `16.10`.
