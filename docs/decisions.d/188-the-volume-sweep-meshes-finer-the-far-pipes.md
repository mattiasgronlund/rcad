## 188. The leaning bar meshes, the volume sweep meshes finer, and the six far pipes are gaps — 2026-09-21

*Three lanes off `f0829431`, dispatched by §175's skill against a base gated
green here first, plus a second round for the lane that did not finish.
§187.4's wrong mesh is fixed at its cause. Along the way, a check the fix added
found a **second wrong mesh** in the fuse fuzz corpus. That one is refused by
name, not yet fixed at its cause. `Model::validate` no longer refuses the LED
disc for its own coarse chords, but **none of the 20 LED files imports yet**.
The six far-pipe refusals §187.3 could not classify are **gaps**, and the
witnesses are recorded. The fix is described and measured in a scratch copy
only; the kernel does not have it yet.*

Merged as `0c92b316` (`lane-coarse-sweep` at `7dda6ab5`, `lane-far-pipe-anchor`
at `85dc3160`), `6439dc80` (`ignored_tests` 3 → 4), then `9ff5c79e`
(`lane-leaning-bar-2` at `5f8559e2`, which carries `lane-leaning-bar`'s
`b782b51c`) and `bc0b8317` (`ignored_tests` 4 → 3). `just gate quiet`, run
here: PASS on `6439dc80`, 2137 run, 2137 passed, 4 skipped; PASS on `bc0b8317`,
2139 run, 2139 passed, 3 skipped. Every other number below is a lane's.
`coarse-sweep`'s were re-run by its verifier and matched, down to every stray
distance. `leaning-bar-2`'s verifier re-ran everything except the numbers from
probes that were since deleted, and says which those are. `far-pipe-anchor`
changed only a test file and had no verifier, so its numbers are its own.

### 188.1 The leaning bar: two faces cut off the same ear

For the lean-5 bar at deflection 0.1, the flat cap and the cylinder wall each
triangulate in their own (u, v). Near the tip of the ellipse, both cut off the
same ear, on mesh points 33, 34 and 35. The cap drew `[34,35,33]` and the wall
drew `[34,33,35]`, so side 33–35 was used four times. Each face was right in
its own coordinates. The bad triangle only shows once the two faces meet at
about 11°, which is the lane's own arithmetic and was not run.

The fix is in `rcad-mesh`'s triangulator, not in any clean-up afterwards. On a
curved surface, a side that runs from the corner before a corner to the corner
after it is split at its midpoint, when the middle corner lies inside one
edge. If the side cannot be split, the triangulator refuses with
`CannotTriangulate::ASideCutsOffOneCornerOfACurvedPatch`. Model vertices and
the ends of cuts are exempt, through
`TrimmedDomain::with_corners_between_two_edges`, because an ear there is a real
corner of the patch. The first version had no exemption: 14 tests failed, since
every corner of a rectangular cell was being split.

A second fault showed up in the first round: `cut_a_cap_up_to_its_pole` never
put either end of a quarter cut into `on_a_cut`. The rule then took the cut's
corner on the cone's base circle for a corner inside one edge. It split a
triangle that was legitimate, and the pointed cone ran out of room at 1e-4.
With both ends marked, the cone meshes at 1e-4 in 2052 triangles.
§187.4's first version therefore also over-refined every cone and ball cap:
23 134 triangles at 0.002, against 458 now.

`a_bar_whose_top_leans_five_across_for_one_up_meshes_into_one_closed_surface`
is no longer ignored. `a_bar_leaning_anywhere_from_two_to_twenty_across_for_one_up_uses_every_side_twice`
sweeps leans from 2 to 20 at seven settings. Both fail with `mesh.rs` and
`rcad-mesh/src` put back to base; the verifier reproduced that.

### 188.2 The second wrong mesh: a Pole step that is not at the pole

`BuildingAMesh::finish` now checks that every side the triangulator drew
inside a patch is used exactly twice. On its first run it fired on a corpus
model with the triangulator rule switched off, so the check found an older
wrong mesh rather than causing one. The model is the first half of fuse seed
`3482a32e1c9186370acb804a509b12d5cbaf20a7`. It is one sphere face of radius 5,
and its only wire runs out along one edge and back, with a `Pole` step at each
end. One vertex sits at the pole. **The other sits at the sphere's centre**,
5 from either pole, so it is not on the surface at all. The walk trusts the
`Pole` step and squashes the whole equator into one mesh point. The two quarter
patches either side of the slit then each draw `(0,1,2)` and `(1,2,3)`, facing
opposite ways, so side 1–2 is used four times. The fold happens in the walk,
before the triangulator's rule sees anything.

`finish()` now returns `Error::TwoPatchesOfTheMeshDrewTheSameSide`, naming the
lowest-numbered bad side so that the same model is always refused the same
way. A panic was wrong here: this is caller input that `validate` let through,
not a broken internal invariant. The lane said no other corpus model hits the
new error, but it ran that sweep with a probe it later deleted, and the
verifier could not repeat it.

**The cause is refused, not fixed.** Nothing in `Model::validate` checks that a
`Pole` step's vertex stands where the surface pinches. The mesher cannot check
it for `validate`, because `validate` meshes with room `f64::MAX`. An exact
check would be wrong the other way: the kernel's own balls in this corpus put
their pole vertices up to 3.6e-15 off the pinch. The check belongs in
`validate.rs`, against the same tolerance it uses for "a vertex lies on its
face". No unit test builds such a ball by hand yet. The corpus test is the one
that fails without the change.

### 188.3 The volume sweep meshes finer where its own chords cross

`how_much_space_a_curved_solid_encloses` still meshes at
`as_coarse_as_a_mesh_gets()` first. If that mesh is refused with
`BoundaryCrossesItself`, `CornersCoincide` or `LoopsTouch`, the new
`the_coarsest_mesh_whose_sides_do_not_cross` tries half the width of the
solid's box, then keeps halving. It stops below half the distance between the
two closest distinct corners, or at the mesher's own limits, and then returns
the coarse refusal. A finer mesh refused for any other reason returns that
refusal, because it is the solid's real next blocker. Only a mesh the
triangulator accepted is ever used, so where the halving stops changes the
cost and never the answer. A fixed constant was ruled out because every
constant has a solid it fails: a hole half as far from the rim.

The corner floor is a cost bound, not a proof. A valid solid whose wall is
thinner than half its closest pair of corners would still be refused, under
the coarse mesh's name, and the doc comment says so. The first floor was the
model's linear tolerance. On `builder.rs`'s tube with a hole as wide as its
wall, that one took 16.6 s and failed; with the corner floor, the same test
takes 0.010 s.

**This floor is argued for chords that cross, not for folds** (2026-10-07). A
chord at deflection `d` strays at most `d` from its curve, so two pieces of
boundary the solid keeps apart by more than `2d` cannot be made to cross by the
mesh, and that is linear in the gap. A fold (`TwoPatchesOfTheMeshDrewTheSameSide`)
is a different sampling artefact: its side joins two points the two faces
share, so it spans the gap between their shared curves, and it goes once a
patch takes a point inside instead — which `rcad-mesh` does once the side's
charge `L²/(3ρ)` is over the deflection, so the fold goes once `d < L²/(3ρ)`,
quadratic in the gap. Since Q1198 (`c3a2a3f2`) a fold is asked down to its own
floor in `validate.rs`'s `the_coarsest_mesh_whose_sides_do_not_cross`,
`closest²/(6ρ)` and never above the corner floor; the function's doc comment
argues it under "What stops a fold". The ring that needed it is
`a_cone_less_a_whole_ball_pushing_out_through_its_side` (`2√2π/3`): both its
faces drew the same side from circle to circle at the corner floor and only
folded apart once the ball's band took a point inside.

**2026-10-14, Q1973:** that floor is also never below half the box's width
over `2¹³` (`HALVINGS_A_FOLD_IS_ASKED_DOWN_TO` in `validate.rs`), a cost bound
set from what `rcad-topo`'s tests and the slow tier clear. The doc comment of
`the_coarsest_mesh_whose_sides_do_not_cross`, under "What stops a fold",
argues it.

**The LED files, after the fix** (the kicad rate is 114/323 either side, and no
pin moved):

| What the 20 files stop on now | Files |
|---|---|
| the quartic marcher's 50 shortenings | 7 |
| a side strays further than the deflection allows (1.04 to 1.78) | 6 |
| too nearly parallel to follow | 5 |
| a wire will not lie flat in the face's own two coordinates | 2 |
| the boundary crosses itself | 0 |

This differs from §187.2's prediction, which came from a probe at a fixed
deflection of 0.01. "Placeable only within 21.33" does not appear at all, and
the strays are not 1.667. The retry stops at the first deflection whose chords
do not cross, which is far coarser than 0.01. Two files,
`LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm` and `_Z6.0mm`, stop on "will not lie
flat", which nobody predicted. Whether that too comes from the coarse mesh is
unmeasured.

### 188.4 The six far-pipe refusals are gaps

§187.3's six refusals are three pairs, each refused in both argument orders.
The anchor is not the cause. For each pair and each of the kernel's three
exactly-square normals, the lane searched every f64 offset. It halved over the
f64s in order and checked each offset against the exact true lines. The offset
that holds both lines and the offset whose reverse holds both lines are always
neighbours, so no f64 offset separates them. **Within the kernel's family —
those three normals, any f64 offset — it is a demonstrated limit.**

The answer still exists. Add one of the three normals to another, scaled by
±2^p for p from -80 to 80. Where that sum rounds nothing, the result is
exactly square to both pipes, and for each pair such a normal separates the
lines at exactly one f64 offset. That makes the six refusals gaps by §16.10.
The witnesses are in
`three_far_pairs_the_sweep_refuses_have_a_plane_between_their_lines`, and the
sweep now fails if a refusal has a separating offset along one of the kernel's
own normals.

The fix, **measured only in a scratch copy of the crate**, has two halves in
`crates/rcad-geom/src/intersect.rs`:
(1) in `picking_normals_to_try`, add those power-of-two combinations, keeping
only the ones `Exact::dot_of` says are square to the pipes; (2) in
`two_lines_between_two_pipes`, read each normal's middle exactly and try the
f64 offsets from 2 ulps below its enclosure to 2 ulps above.
Half (1) alone leaves 2 refused, half (2) alone leaves all 6 refused, and both
together answer 8000 of 8000 with 0 wrong. Nothing has timed it, although it
only runs on the path where the first try has already refused.
`the_kernel_names_the_three_far_pairs_it_refuses` is `#[ignore]`d until the
fix lands.

### 188.5 What is left

- **`validate` passes a `Pole` step whose vertex is off the pinch**, §188.2.
  The mesher refuses it after the fact. It first needs a hand-built test ball.
  This is `validate.rs`.
- **The far-pipe fix**, §188.4, in `intersect.rs`: make it, time it, and
  un-ignore its test.
- **The sweep half of §187.1**: `validate` passing a flat-only bent edge.
  Also `validate.rs`, so it cannot run beside the first item.
- **The two LED files that will not lie flat**, §188.3. The other 18 are
  behind named refusals that each have a section of their own.
- **The turned-bend band**, §187.4 item 2, and **fuse::bake's corner test**,
  the other two ignored tests.

Commands: `just decision 187.4`, `187.2`, `187.3`, `16.10`; the `lanes`
workflow runs `wf_110520e6-de9` and `wf_715b0895-6dc`; `git merge
lane-coarse-sweep lane-far-pipe-anchor`, `git merge lane-leaning-bar-2`;
`just gate quiet` on `6439dc80` and `bc0b8317`.
