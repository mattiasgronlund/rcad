## 201. The pointed cone's skin holds its own half, the far pole side ends where its cut leaves, and a point the ray and the plane disagree about is dropped — 2026-09-22

*One batch, dispatched by §175's skill off `055aa4a9`: workflow run
`wf_d566121f-c31`, three Opus lanes. `055aa4a9` was gated here before the batch:
PASS, 2251 run, 2251 passed, 6 skipped. **The disk filled at about 14:33, during
the batch.** Two of the three lanes died with their fixes committed but before they
reported, and neither got a verifier. The third lane's verifier died while it
was running the gate. So the check a verifier makes was made here, by hand, on
the merge: each lane's `src/` put back to `055aa4a9` and its tests run (201.4).
`just clean-targets` freed 46G. `just gate quiet` run here on the merge
`4514eff4`: PASS, 2254 run, 2254 passed, 6 skipped. A number below is a lane's
unless it names a command run here.*

### 201.1 A pointed cone's skin holds its own half — and the box still stops half way up

`lane-cone-classify` (`6d6461c3`, `cc494235`) takes §200.4's first item half
of the way. The pointed cone's skin has one corner, on its cap.
`the_side_this_face_lies_on` reads only corners off the cap, found none, and
refused every point asked of the face: 3444 refusals over §200.3's grid, all at
`z ≥ 4.03`. And once it could say, the side of the cap alone would also have
held the half of the cone mirrored through the apex. `holds.rs` now has a cone
face keep to the nappe its corners are on. A cone face trimmed by one closed
curve round the axis is now the piece that reaches the apex.

The grid from half way up (`z > 4`) now answers 876 inside and 9749 outside,
none wrong and none refused (`a_pointed_cone_knows_its_inside.rs`, pinned).
`a_plane_cut_along_a_cones_other_nappe.rs`: the family of pointed cones with a
corner-up cube moves from 0 right and 40 refused to 16 right and 24 refused.

**Below `z = 4` every point inside the cone still answers `Outside`, and that
is a wrong answer on `main`.** `box_the_surface_bounds` (`view.rs`) bounds a
cone face by the box round its corners' shadows on the axis. The apex is not a
corner, so the box stops at `z = 4`, and `classify_point` reads the box before
it fires a ray. §200.3's three named points are all below 4. The same box
decides, at `overlap.rs:159`, that two solids whose boxes miss do not touch. That
matches §200.3's wrong fuse answer exactly — nothing cut, both solids back
untouched — but it is **not shown yet**. `view.rs` was outside this lane's
write set, so it is the next lane's (201.5).

### 201.2 The far pole side ends at the meridian the patch leaves by

`lane-far-pole-quarter-turn` (`e26abef6`) closes §200.4's third item. In
`cut_a_cap_up_to_its_pole`, each patch's side at the far pole ended a quarter
turn back from the meridian it came up by. On the dome of the turned ball and
block the four cut corners are 2.74, 0.40, 2.74 and 0.40 rad apart, so in two
patches that side overshot the next cut's meridian by 1.17 rad and folded back
on itself (`BoundaryCrossesItself`; the lane's word, from a probe since
removed). It now ends at the patch's own first corner.

§191's pin is an answer test now, renamed
`a_turned_ball_united_with_a_block_holds_what_the_upright_one_does`. It
checks the turned union against the upright one, each within its own
`volume_error` bound: 126.82722 against 126.82571, a difference of 0.0015, and
a bound of 3.98 (the lane's word). The 63 caps of
`a_cap_whose_rim_runs_through_the_other_pole.rs` still pass. They cannot tell
the two rules apart, because their cut corners sit on the quarters, so no cap
case was added. Its verifier saw the new test fail on base `mesh.rs`.

### 201.3 A point the ray and the stored plane disagree about is dropped

`lane-flat-half-disk-sliver` (`d12913d4`) closes §200.4's second item. It was
a gap. In the 90° wedge cut by two knives, the first candidate point for the
half disk of `x = 0`, `(0, 1.1e-16, −0.5)`, is certified Inside by the ray.
**Both signs were certified, but they were about two different faces.** The
ray fans the knife's corners `(−6, 8.9e-16, z)` and `(6, −8.9e-16, z)`, whose
triangles hold the axis exactly. The stored plane the arrangement cut along
misses the axis by 4.4e-16. `select.rs` now drops a candidate when, on a flat
face with straight sides, the stored plane and a fan triangle give opposite
decided signs for it. There is no tolerance: it takes two decided, opposite
signs.

That let the union reach `rings_round` (`assemble.rs`) with a half-disk hole of
two corners, which winds round no area, and it panicked. A ring enclosing
nothing is now a hole when another ring is the outline, and the outline when it
is the only one.

`a_narrow_lune_from_a_pole_corner.rs` re-pinned: 12 more cases cut by two knives
answer, validate and measure. These are 90° at every radius, and 10° at radii
0.001 and 1000, each for both the intersection and the union.

**The match the lane added ended in `| _`, and `check-wildcards` would have
counted it.** The lane's uncommitted answer, left in its tree when the disk
filled, turned the `match` into `==` comparisons. That hides the arm from the
count without removing it, which is what §40 is against. It was not taken:
`4514eff4` names `Zero` and `Undecidable` and panics on any variant nobody here
knows, which is the form `cc494235` gave `holds.rs`.

### 201.4 What was checked here, in place of the verifiers

On `4514eff4`, each lane's source put back to `055aa4a9` with `git restore
--source`, then restored:

- `holds.rs` at base: `just t -p rcad-topo --test merged -E 'test(/^a_pointed_cone_knows_its_inside::/)
  or test(/^a_plane_cut_along_a_cones_other_nappe::/)'` — 9 run, 3 failed.
- `fuse/` at base: `just t -p rcad-topo --test merged -E 'test(/^a_narrow_lune_from_a_pole_corner::/)'`
  — 5 run, 1 failed (the two-knives wedge test).

The flat-half-disk lane also says a test pins the disagreement itself. That test
passes on base, as a statement of fact about the geometry should.

### 201.5 What is left

- **The pointed cone's box stops half way up (201.1).** First, because it is a
  wrong answer. Taken by `lane-cone-box-reaches-apex` in workflow run
  `wf_e20ea19a-0a0`, off `4514eff4`. That lane also owes §16.10's demonstration
  for the pipe arm's claim beside it: that a face's corners settle how far along
  the axis it reaches. The claim is silent about a side that is a single closed
  ellipse.
- `add_ball` plus a cut through a pole refused `CannotTellTheCellsOfAFaceApart`
  (§200.2). Taken by `lane-ball-pole-cut` in the same run.
- `mesh.rs:839`, the piece of a ball holding both poles, and the named refusal
  for a closed edge that neither direction rule answers (§198.3). Taken by
  `lane-ball-both-poles-mesh` in the same run.
- The fuse cutting along a cone's other nappe, which §200.3 found refused 224 of
  224 times.

Commands: `just decision 200.3`, `200.4`, `198.3`, `40`, `16.10`.
