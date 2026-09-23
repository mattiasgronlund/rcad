## 209. The cone's tip charged whole, the apex ring walked, and the four tests that moved

§208 left one thing blocking everything behind it: `measure.rs` gave a cone face
reaching its apex a smallest radius of normal curvature of nothing, so every
solid holding an apex published a `volume_error` and an `area_error` of `inf`.
That is honest and useless — a measured volume with an infinite bound cannot be
held against anything, and a test that compares one against the other is a test
that cannot fail. Two lanes close it and the second was held behind the first.

### 209.1 A cone's tip is charged its whole size, and the rest is charged in bands

`what_a_cone_tip_costs` splits the face at `v = 2·deviation·√(1 + slope²)/slope`,
charges the tip below the split its whole area and volume —
`4π·d²(1 + s²)^{3/2}/s` and `(8π/3)·d³(1 + s²)^{3/2}/s` — and argues everything
above it off a real radius. The tip needs no offset surface to argue from, which
is the point: the offset cone does not exist down there, and no ordering of the
old argument was going to reach it.

Two things the doc comment that predicted this fix had wrong, and both were
found by running it rather than reading it — the memory note "a named fix in a
section may be unmeasured" is this, again:

1. **The split runs through the triangles, not between them.** A cone is
   developable, so the mesher fans a pointed cone from its apex and *every*
   triangle of that face has a corner at the apex. Sorting whole triangles
   either side of the split puts the whole face below it.
2. **One radius for everything above the split does not converge.** The factor
   at the split radius has no `d` in it, so the area bound sat still as the mesh
   got finer. The face is cut into bands of radius instead, each doubling on the
   last, each charged off its own floor — `O(deviation)`, the order a ball's
   bound already has.

Measured on a bare pointed cone, slope 0.5, height 8: volume
`134.02461503321067` against a true `128π/3 = 134.0412865531645`, and a bound of
`inf` before and `0.1501600338829092` after at a deflection of `0.002`. It
quarters as the deflection does — `0.6019853760349294` at `0.008`,
`0.1501600338829092` at `0.002`, `0.037682914581960984` at `0.0005`.

The tip is charged the cheaper of its whole size and the plain bound, so no
bound anywhere gets looser: a truncated cone whose near cap stops just short of
the split still has a real radius, and taking the tip's whole size there would
loosen a bound already earned.

### 209.2 A ring of arcs round an apex is walked, and a ring is a cap only when it is alone

`lane-cone-apex-ring` closes the mesher half. A pointed cone inside a cube stood
on its corner keeps a piece of nappe round the apex bounded by three arcs, one
from each of the cube's upper planes — a ring with no seam and no pole, which
`may_come_back_a_whole_turn_on` now admits.

The second commit is the limit on that, and it is the more interesting half. A
frustum's band between its own near cap circle and a ring of arcs from the same
cube is **also** a ring with no seam, and letting it through sent
`cut_a_ring_across` a pair it could not cut: the fuse answered and the measure
came back `CannotMeshAFace { why: BoundaryCrossesItself }`, where before the
face was refused outright. A face with a hole that a ring also bounds is a
different shape with a different argument to make — which of the two pieces the
ring cuts the surface into does the hole sit in — and nothing in the walk says.
So the arm asks for the face's **one** wire as well, which is the shape §206.6
names and nothing else.

That is why §200.3's eight fuses all answer now while the frustum's twenty-four
still refuse, and both are pinned:
`a_corner_up_cube_centred_on_a_pointed_cones_apex_answers_every_rule_right` and
`a_corner_up_cube_round_a_frustums_apex_answers_right_or_refuses`.

### 209.3 Four tests outside the write sets moved, and one only changed stage

Three moved the way the fixes intend. `a_plane_cut_along_a_cones_other_nappe.rs`
goes from 20 right and 20 refused to 40 right, and from 4 and 4 to 8; both tests
are **renamed**, because `_or_refuses` and `_answers_half_and_cannot_mesh_the_rest`
describe a body that no longer exists, and this repo does not keep a name over a
changed body. `a_solid_with_a_cone_face.rs` and
`a_cone_and_a_torus_through_the_front_door.rs` each argued at length that an
infinite bound was the honest answer; that argument is now wrong, so the prose
above each was rewritten and not only the assertion.

`a_pointed_cone_knows_its_inside.rs`'s slack guard moved from a five-hundredth
of the volume to a hundredth. The guard exists so the comparison cannot go
vacuous again, and the five-hundredth was sized against the union alone: measured,
the union's slack is `0.20872487972302164` on `193.64491337804006` and the
intersection's `0.018084103879991164` on `4.39637317512444`, so a five-hundredth
passes one and fails the other while both are right. The intersection is the
tighter case because it is almost all apex — the tip the bound charges whole is
nearly the whole of what the two solids share, where in the union it is a crumb
of a much larger body. A threshold that splits those two is measuring the shape,
not the check.

**The fourth is pinned as measured and not as understood, and that is on the
record rather than dressed up.** `fuses_that_touch.rs`'s ball resting on a block
it overhangs is refused before and after — no wrong answer appeared where there
used to be none — but the name moved from `AWireWillNotLieFlatOnItsFace` to
`ACurvedFaceGoesRightRoundWithNoSeam`. That name is raised in exactly one place,
`Arrangement::the_whole_surface`, reached when a face's arrangement holds no arcs
at all, and refusing there for every surface but a ball. The face it names is in
the fuse's **own working model**: `Model::face` on it answers
`KeyFromAnotherModel`, and the provenance log for the fuse comes back empty. So
nothing outside the kernel can say which surface that face is on, and whether
the name fits it is not something this session could demonstrate either way.
§16.10 asks for a demonstration and there is none, so no claim is made.

What would settle it is a lane inside `rcad-topo` with the working model in
hand: name the surface, and if it is a plane, say why a fuse built a plane face
with no arcs on it — `the_whole_surface`'s own doc comment argues that reaching
it means every wire was a seam, and that argument was written about the caller's
model, not about one the fuse builds for itself.

### 209.4 What is left of the batch, unmerged and on disk

Two lanes of the second batch are not merged and neither is ready:

- `lane-one-closed-curve-test` (`7e541a74`) — verdict `fix_first`. The predicate
  work is good and the verifier ran the differential itself, but the brief's
  premise was wrong and the lane proved it: the nappe floor did not move,
  9326 → 9326, because `the_way_into_an_arc` asks
  `the_point_a_circle_cut_by_a_plane_is_centred_under` first and that needs a
  **circle**, not merely a closed curve, so an ellipse arc returns `None` before
  any widened predicate is asked. It also breaks
  `a_block_leaning_through_a_frustum_is_cut_along_the_cone_far_above_the_frustum`,
  and the verifier showed the widening is the cause. Whether the new
  hyperbola-wall behaviour is right is a correctness question and not a count to
  bump. The real next lane is the one that report names: give
  `the_way_into_an_arc` an argument for which side of a chord an **ellipse** arc
  lies on. An ellipse's centre is constructible — where the plane meets the line
  through the apex conjugate to the plane, *not* where the axis meets the plane —
  and nothing in `rcad-geom` builds it yet.
- `lane-fold-repro-hunt` (`c64b050b`) — no repro, and `limit` for the family it
  searched. Test-only, and `verify` was off by the dispatching session's choice,
  so nothing cross-examined it. It also reports that §208.3's brief overstated
  the family: the code asks `piece.closes_on_one_curve`, which is weaker than
  "cut in two closed curves", so a sliver chopped by further planes qualifies
  too. Its proposed next build — chop with two extra planes so each new flat
  face has three corners — is unbuilt, and §208.3's fold-retry arm still has no
  test that fails without it.

§208.3's arm was merged knowing that, deliberately and on the record. It stays
merged: reverting six lines that are right costs more than carrying them until
the family that exercises them is built, and §208.3 already says which six.

### 209.5 A limit that rests on someone else's gap is not a limit

`lane-fold-repro-hunt` is not merged for a second reason, found reading it here
and worth more than the first. Its argument runs: for a coarse mesh to enclose
zero every point sits on one plane, so both bounding curves lie in that plane,
so the surface is a torus square to its axis, so the shape is the crown of a
ring — and the crown of a ring cannot be meshed, so the family is a **limit**
under §16.10.

The last step is the one that does not hold. §16.10 draws the line at whether
the *answer exists*: a limit is when it does not, a gap is when it does and the
arithmetic could not reach it. The crown of a ring is an ordinary solid — the
lane builds it through `SolidBuilder` and the shell closes, with 20/5/4 chosen
as a Pythagorean triple so both corners are exact on the quartic. What refuses
it is `ADraftFaceCouldNotBeMeshed { why: CornersCoincide { position: 4 } }`,
because the face goes right round the **ring** while a walked loop carries one
wrapping coordinate and `Torus::point_at` spends it on the tube. That is a gap
in the mesher, not the absence of an answer.

So the family's verdict is conditional on a defect elsewhere, and it evaporates
the day that defect is fixed — which is not a thing to write into the tree as a
limit. §208.4 is the warning: a ring of arcs with no seam was unwalkable until it
was walked, and nothing in the argument for refusing it had been cone-specific.
The same widening is generic over any surface that wraps.

What the branch does have is a good demonstration, and it should keep it: the
crown of a ring, built exactly, refused by name. The right shape for it is a
test pinning that refusal as a **gap** with the mesher's own reason, plus the
sliver family the lane says the code's `piece.closes_on_one_curve` actually
admits and it did not sweep. Then §208.3's arm has something that fails without
it, which is still what it is missing.
