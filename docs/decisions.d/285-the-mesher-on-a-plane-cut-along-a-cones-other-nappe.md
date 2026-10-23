## 285. The mesher on a plane cut along a cone's other nappe: two quarter cuts from one corner, and two faces that fold — 2026-09-27

*Queue row Q124. Lane `lane-q124-other-nappe-mesher-2`, off `ff78f99f`, taking
over the WIP commit `ecb07bb2` of a lane the usage limit killed. §251.6 left
the mesher's refusals in `a_plane_cut_along_a_cones_other_nappe.rs` unread.
**There are eighteen, and they have two causes.** Sixteen are a cap of the
pointed cone that the mesher cuts into four quarters, where two of the four
cuts leave from the same corner. Two are the frustum's face and the block's
steep face, which share two arcs, and both draw the side between the arcs'
middles. **Both are gaps.** A change to the corner choice answers 12 of the
16, and retrying a fold answers the other two. The last 4 of the 16 are a face
meant to pass through the apex, which misses it by a rounding. They need a
third change, and that one is only half found. No code changed here. It is
pinned in `crates/rcad-topo/tests/meshing_a_plane_cut_along_a_cones_other_nappe.rs`.*

### 285.1 The count, and where it stops

Recounted on `ff78f99f` by
`the_mesher_refuses_sixteen_caps_of_the_pointed_cone_and_two_folds_on_the_frustum`,
over the 96 fuses of the pointed-cone family and the 32 of the frustum family
whose apex is inside the block:

- **16 `CannotMeshAFace { why: CornersCoincide { position: 2 } }`**, all
  pointed cone. They are eight blocks, each by the intersection and by the
  block less the cone. The blocks are lean 1/4 and 3/8 at crossing 0, turn 1;
  and lean 3/8 at crossings 1/8 and 1/2, turns 0, 1 and 3. Q124's row says 12.
  §282.5 brought 4 of the old point-in-piece refusals this far, so it is 16.
- **2 `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }`**: the
  frustum's intersection with lean 3/8, crossing −1/4, at turns 3 and 3.5.

Only the intersection and the block less the cone refuse. Those are the two
rules that keep the piece of the cone **inside** the block, so every refused
face is that piece or borders it.

All eighteen stop in the check the fuse makes before it writes a curved answer.
`how_much_space_it_encloses` (`fuse/assemble.rs:2321`) calls
`how_much_space_a_curved_solid_encloses`, which calls
`the_coarsest_mesh_whose_sides_do_not_cross` (`validate.rs:1872`). That meshes
at `as_coarse_as_a_mesh_gets`, where each crossing is walked as two chords.
That function retries only a refusal whose sides cross. `CornersCoincide` is
kept out of that on purpose (`its_sides_cross`'s doc comment), and a fold is
handed straight back at 1877. So neither refusal gets a finer mesh.

### 285.2 Sixteen caps: two quarter cuts from one corner

**Which face.** The piece of the cone inside the block contains the apex and
goes right round it. The steep face's crossing runs round the far side of the
apex, and the rim closes the loop on the block's side. So it is a cap, and
`cut_up` sends it to `cut_a_cap_up_to_its_pole` (`mesh.rs:1103`). That function
cuts a cap into four patches, each a quarter of a turn wide. Each cut runs up
to the apex from `nearest_corner_off_the_rims_poles` (`mesh.rs:1126`, `2126`),
which is the rim corner nearest that quarter.

**What it was handed.** With a print in the function (285.6), lean 3/8,
crossing 1/2, turn 0, intersection:

```text
cap climbed −2π, way −1, quarters [0.791, −0.780, −2.351, −3.921], along [0, 2, 3, 3]
  corner 0: [0.791, 8]     rim, one end
  corner 1: [0.000, 8]     rim, middle
  corner 2: [−0.791, 8]    rim, other end
  corner 3: [−π, 0.214]    the crossing's middle
refused: CornersCoincide { position: 2 }
  patch corner 0: [−π, 0.214]   corner 3
  patch corner 1: [−π, 0]       the apex
  patch corner 2: [−π, 0]       the apex again
```

The kept rim is `|θ| ≤ 0.791`, where `cos 0.791 = (3/8 · 7.5)/4`. The crossing
takes the other 4.70 radians of the turn and carries one corner, at its middle.
Quarters 2 and 3 are both nearest that corner, 0.790 and 0.780 away. So
`along[2] == along[3]`, the patch between them is corner 3 alone, and it closes
with a cut of one step to the apex and the apex twice at one longitude. The
triangulator refuses the second apex.

**Both legs of the killed lane's hypothesis hold, separately.** The first leg is
that the refused face reaches the call. The print is made inside
`cut_a_cap_up_to_its_pole`, and it names the same face, `FaceSlot(2v1)` of the
answer's model, as the refusal. The second leg is that the four corners chosen
are not all different. They are `[0, 2, 3, 3]` here. They are `[0, 1, 2, 2]` for
lean 1/4, crossing 0, turn 1, and `[0, 2, 3, 3]` for lean 3/8, crossing 0,
turn 1. In each, the refused patch is a corner followed by the apex twice.

**The shape predicts which blocks refuse.** The walk gives four corners: both
ends of the kept rim, the rim's middle, and the crossing's middle. The crossing's
middle is half a turn from the rim's middle. Call half the kept rim `α`. Cuts
start from one end of the rim, a quarter turn apart, and two of them land on
the crossing's middle **exactly when `α < π/3`**. Then the crossing is more
than two thirds of the turn and holds two quarters. Lean 3/8 keeps `α = 0.739`
and `0.791`, so it refuses at every turn. Lean 1/4 keeps `1.056` and `1.083`,
so it refuses at none.
`two_quarter_cuts_share_a_corner_exactly_when_the_kept_rim_is_under_a_third_of_a_turn`
checks the prediction against all twelve fuses. At lean 1/4, crossing 0,
`α = π/3` exactly, and the tie is broken by rounding.

**A gap, shown.** The cap exists, and a cut that leaves from four different
corners reaches it. Experiment A (285.6) picks each corner after the first
from those later in the loop, leaving room for the rest. Under the family's
four checks, **12 of the 16 answer right**. They validate, have one solid,
decide the grid exactly, and measure at 0.002 within the bound. Those 12 are
every block but crossing 0. The other 4 go on to 285.3.

### 285.3 Four of the sixteen: a face meant to pass through the apex

At crossing 0 the block's steep face is meant to pass through the apex. Its
placement is rounded, so it misses. At turn 1 the fuse's crossing of that face
and the cone passes just above the apex. The print put the crossing's middle
corner at height `6.1e-16` for lean 1/4 and `2.7e-16` for lean 3/8. So the
piece inside the block goes right round the apex, and it is a cap like the
other twelve.

**Which side of the face the apex is really on does not decide it.**
`which_side_of_the_face_meant_through_the_apex_it_is_on_does_not_say_which_refuse`
classifies the apex against the block at both leans and turns 0, 1 and 3. The
block puts the apex inside at three of the six, outside at two, and on its
boundary at one. The two that refuse are one outside (lean 1/4) and one inside
(lean 3/8). What makes the cap is where the fuse's crossing came out, not the
exact side.

**With experiment A they get one step further, and meet a second gap.** All
four answers hold exactly the right grid points. Lean 3/8 decides 27 214 of
them for the intersection and 21 610 for the block less the cone. But the finer
measures refuse:

- lean 1/4: it measures right at 0.02, and at 0.01, 0.005 and 0.002 it refuses
  `BoundaryCrossesItself`;
- lean 3/8: it refuses `ASideCutsOffOneCornerOfACurvedPatch` at all five
  rungs. That refusal's doc comment says it had never been seen in this
  workspace.

The refused face is the cap again. The print names it, and it is on the cone.
Which of its four patches refused was not traced. The shape says what is
unusual about it. The crossing's stretch round the apex is `2π − 2α` long:
4.19 radians for lean 1/4 and 4.84 for lean 3/8. The whole stretch is within a
few `1e-16` of the apex. The quarters land two (lean 1/4) or three (lean 3/8)
of the four cuts on it. So at least one patch lies wholly between that stretch
and the apex: a strip a quarter of a turn long and about `1e-16` tall in the
cone's own coordinates. A ball has a rule for this, and a cone does not.
`the_pole_a_point_sits_within_a_rounding_of` (`mesh.rs:3556`) reads a point
within a rounding of a ball's pole as the pole (§192.1, §193.3). It answers
`None` for a cone. Experiment C adds a cone arm: within one
`POINT_ALONG_A_CROSSING_ROUNDOFF` of the apex is the apex. With A and B as
well, **the two blocks-less-cone answer right**. The two intersections are
right on the grid, but every measure refuses
`TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }`. That fold was not
traced.

**Verdict: a gap.** The solid exists, and the fuse's answer holds exactly the
right grid points. A cone apex within a rounding is the same case a ball's pole
already has a rule for. Two of the four measure right with that rule. The other
two show the rule is not all of the fix.

**2026-10-04:** Q846 (Q441's branch, cap cuts leaving only from corners whose
meridian meets the rim nowhere else) answers the two lean-3/8 caps at crossing
0, turn 1 of `meshing_a_plane_cut_along_a_cones_other_nappe.rs`, without
experiment C. Their old first cut ran down the ruling the rim comes up by,
along the rim's own last side, which is not this section's quarter-turn strip:
cuts still land on the near-apex stretch and the mesh answers. The lean-1/4
pair now refuses `ASideCutsOffOneCornerOfACurvedPatch` at 0.01, 0.005 and
0.002, not `BoundaryCrossesItself`
(`mise exec -- just t the_eighteen_held_to_the_familys_four_checks --nocapture`).

### 285.4 Two folds on the frustum

For lean 3/8, crossing −1/4, turn 3, the intersection has four faces. Two are
flat pieces of the two caps. One is the frustum's face between the two arcs
where the steep face crosses it. One is the steep face's own piece. The steep
face and the frustum's face share **both** arcs. At the coarsest mesh each
arc is two chords, with one walked point between them. The two points sit at
`z = 3.893` at `θ' = ±0.646` from the block's middle, one the mirror of the
other. With a print in `finish` (285.6), the folded side is the one between
those two points:

```text
folded side (1, 3): (−1.704, −0.942, 3.893) to (−1.373, 1.380, 3.893)
  cone face  (3v1): triangles [0, 1, 3] and [2, 3, 1]
  steep face (4v1): triangles [4, 3, 1] and [7, 1, 3]
```

The cone face's loop is `[5, 0, 4, 1, 7, 2, 6, 3]` and the steep face's is
`[4, 5, 3, 6, 7, 1]`. Neither loop has 1 and 3 next to each other, so for
each face the side 1–3 is drawn inside the patch. Each draws it twice, which
is four uses. The comment above `between_two_edges.push` in `hand_over`
(`mesh.rs:1628`) names this case: two edges between the same two faces are
left for `finish` to catch.

**A gap, shown.** A finer mesh gives each arc more points, and the two faces
stop picking the same diagonal. Experiment B retries a fold in
`the_coarsest_mesh_whose_sides_do_not_cross`, the same way that function
retries a crossing. With it, **both answer right** under the four checks. §208.3
added the same retry to the zero-enclosure ladder and found that no solid used
it. These two are the first solids found whose **coarsest** mesh folds.

### 285.5 Against Q119

§280's builder refusal (Q119) was `TwoPatchesOfTheMeshDrewTheSameSide
{ drawn: 1, used: 1 }`. It was a chart fault: the mesher took a torus's ring
angle straight from `atan2`, so a wall past `π` folded back on its own seam
(§280.5). Q190 closed it (§280.8). **None of the eighteen here is a chart
fault.** The cone's two coordinates are unwrapped correctly in every patch
printed. All three causes are about where the mesher puts its own cuts and
chords on a coarse walk: which corner a quarter cut leaves from, a coarse fold
that is not retried, and an apex within a rounding that is not read as the
apex. What they share with §280 is the verdict: the solid is right, and it is
the mesher that stops it. The `{ drawn: 4, used: 4 }` here is two patches that
each draw one side twice. §280's `{ 1, 1 }` is one patch folded over its own
seam. They are different faults.

### 285.6 The experiments, and how to run them again

All against `ff78f99f`, in `crates/rcad-topo/src/`, outside this lane's files,
so none is committed. Apply one or more, set its variable, and run
`mise exec -- just t -p rcad-topo --test merged -E 'test(=meshing_a_plane_cut_along_a_cones_other_nappe::the_eighteen_held_to_the_familys_four_checks)' --no-capture`.
That test's assertion is this tree's result, so it fails on purpose and prints
the tally first.

**A — four different corners** (`mesh.rs`, `cut_a_cap_up_to_its_pole`, in place
of the `let along` line at 1126; `PROBE_DISTINCT`):

```rust
let mut along = quarters.map(|at| nearest_corner_off_the_rims_poles(cap, at, pinch));
if std::env::var_os("PROBE_DISTINCT").is_some()
    && along.windows(2).any(|pair| pair[0] == pair[1])
{
    let count = cap.corners.len();
    assert!(count >= 4, "PROBE: a cap of {count} corners");
    for step in 1..4 {
        along[step] = (along[step - 1] + 1..=count - (4 - step))
            .min_by(|&one, &other| {
                (cap.corners[one][0] - quarters[step])
                    .abs()
                    .total_cmp(&(cap.corners[other][0] - quarters[step]).abs())
            })
            .expect("room for the quarters still to come");
    }
}
```

**B — retry a fold at the coarsest mesh** (`validate.rs`, at the top of
`the_coarsest_mesh_whose_sides_do_not_cross`; `PROBE_RETRY_FOLD`):

```rust
let retry_a_fold = std::env::var_os("PROBE_RETRY_FOLD").is_some();
let its_sides_cross = |why: &Error| {
    its_sides_cross(why)
        || (retry_a_fold && matches!(why, Error::TwoPatchesOfTheMeshDrewTheSameSide { .. }))
};
```

**C — a cone's apex within a rounding** (`mesh.rs`, first lines of
`the_pole_a_point_sits_within_a_rounding_of`; `PROBE_APEX`):

```rust
if let Surface::Cone(cone) = stored
    && std::env::var_os("PROBE_APEX").is_some()
{
    let apex = cone.apex();
    return ((at - apex).length() <= one_rounding_among([at, apex, apex])).then_some(0.0);
}
```

**The prints** behind the corners, patches and triangles quoted in 285.2–285.4
(`mesh.rs`, all under `PROBE_Q124` but the loops of every patch, under
`PROBE_Q124_PATCHES`). After `along` in `cut_a_cap_up_to_its_pole`:

```rust
if std::env::var_os("PROBE_Q124").is_some() {
    eprintln!(
        "PROBE cap face {:?} climbed {} way {way} quarters {quarters:?} along {along:?}",
        face.key(),
        cap.climbed
    );
    for (at, corner) in cap.corners.iter().enumerate() {
        eprintln!("PROBE   corner {at}: {corner:?} point {}", cap.points[at]);
    }
}
```

In `hand_over`, after `model_corners`, and in `triangulate`'s `Err(why)` arm
before the `return`:

```rust
if std::env::var_os("PROBE_Q124_PATCHES").is_some() {
    eprintln!("PROBE patch of face {:?} on {:?}", face.key(), stored);
    for (which, one) in patch.iter().enumerate() {
        for (at, corner) in one.corners.iter().enumerate() {
            eprintln!(
                "PROBE   loop {which} corner {at}: {corner:?} point {} at {:?}",
                one.points[at], self.points[one.points[at] as usize]
            );
        }
    }
}
// ...
if std::env::var_os("PROBE_Q124").is_some() {
    eprintln!("PROBE refused face {:?} on {:?}: {why:?}", face.key(), stored);
    for (at, corner) in domain.outer().corners().iter().enumerate() {
        eprintln!(
            "PROBE   patch corner {at}: {corner:?} point {} at {:?}",
            corner_is[at], self.points[corner_is[at] as usize]
        );
    }
}
```

In `finish`, with the `folded` binding widened to
`Some((side, drawn, used))`, before the fold is returned:

```rust
if std::env::var_os("PROBE_Q124").is_some() {
    eprintln!(
        "PROBE folded side {side:?}: {:?} to {:?}",
        self.points[side.0 as usize], self.points[side.1 as usize]
    );
    for triangle in &self.triangles {
        let [first, second, third] = triangle.corners;
        let uses = [(first, second), (second, third), (third, first)]
            .iter()
            .any(|&(from, to)| a_side(from, to) == side);
        if uses {
            eprintln!(
                "PROBE   triangle {:?} of face {:?}: {:?}",
                triangle.corners,
                triangle.face,
                triangle.corners.map(|at| self.points[at as usize])
            );
        }
    }
}
```

The harness test prints all eighteen. The quoted cases are the pointed cone at
lean 3/8, crossing 1/2, turn 0, and at crossing 0, turn 1, for both leans; and
the frustum at turn 3.

| experiment | right | the rest |
|---|---|---|
| none | 0 | 16 `CornersCoincide { position: 2 }`, 2 fold `{4, 4}` |
| A | 12 | 2 measure only at 0.02 (`BoundaryCrossesItself` finer); 2 measure at no rung (`ASideCutsOffOneCornerOfACurvedPatch`); 2 fold |
| B | 2 | 16 `CornersCoincide { position: 2 }` |
| A + B + C | 16 | 2 right on the grid, every measure a fold `{4, 4}` (the crossing-0 intersections) |

### 285.7 What this leaves

- **Cause 1, the corner choice**: a gap with a fix shown for 12 of 16. Fixing
  it without cause 3 turns 4 refusals in `a_plane_cut_along_a_cones_other_nappe.rs`
  into answers that do not measure. That file's `fuse_and_check` panics when
  an answer does not measure. So the fix row has to carry cause 3, or that pin
  has to change.
- **Cause 2, the coarse fold**: a gap with a fix shown for both. Experiment B
  is the whole of it, and it is where §208.3's retry already sits one ladder
  down.

  **2026-09-29, closed by Q221 at `14ba0254`.**
  `the_coarsest_mesh_whose_sides_do_not_cross`
  (`crates/rcad-topo/src/validate.rs`) now retries a fold
  (`TwoPatchesOfTheMeshDrewTheSameSide`) at half the room, the same way it
  already retried a crossing — experiment B, landed. The frustum's two folds
  (lean 3/8, crossing −1/4, turns 3 and 3.5, intersection) answer right under
  the family's four checks.
  `a_frustum_whose_apex_is_inside_the_block_answers_right_or_refuses` moved
  from 22 right to 24.
- **Cause 3, the apex within a rounding**: a gap, half found. Experiment C
  answers the two blocks-less-cone. The fold in the two intersections under
  A + B + C is not traced.
- The gate was not run with any of the three experiments. What else they move
  is the fixing lane's to measure.
